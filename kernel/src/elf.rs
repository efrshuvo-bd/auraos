//! Minimal ELF64 loader for static AArch64 ET_EXEC guests.

use crate::console;
use crate::frame::{self, PAGE_SIZE};
use crate::vm::{self, UserMap};

const EI_MAG: [u8; 4] = [0x7f, b'E', b'L', b'F'];
const ELFCLASS64: u8 = 2;
const EM_AARCH64: u16 = 183;
const PT_LOAD: u32 = 1;
const PF_X: u32 = 1;
const PF_W: u32 = 2;

#[repr(C)]
struct Ehdr {
    e_ident: [u8; 16],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u64,
    e_phoff: u64,
    e_shoff: u64,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

#[repr(C)]
struct Phdr {
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_paddr: u64,
    p_filesz: u64,
    p_memsz: u64,
    p_align: u64,
}

pub struct LoadedElf {
    pub entry: usize,
}

pub fn load(ttbr0: usize, image: &[u8]) -> Option<LoadedElf> {
    if image.len() < core::mem::size_of::<Ehdr>() {
        console::println("elf: image too small");
        return None;
    }
    let ehdr = unsafe { &*(image.as_ptr() as *const Ehdr) };
    if ehdr.e_ident[0..4] != EI_MAG {
        console::println("elf: bad magic");
        return None;
    }
    if ehdr.e_ident[4] != ELFCLASS64 || ehdr.e_machine != EM_AARCH64 {
        console::println("elf: not aarch64 elf64");
        return None;
    }

    let phoff = ehdr.e_phoff as usize;
    let phentsize = ehdr.e_phentsize as usize;
    let phnum = ehdr.e_phnum as usize;

    for i in 0..phnum {
        let off = phoff + i * phentsize;
        if off + core::mem::size_of::<Phdr>() > image.len() {
            return None;
        }
        let ph = unsafe { &*(image.as_ptr().add(off) as *const Phdr) };
        if ph.p_type != PT_LOAD {
            continue;
        }
        if !load_segment(ttbr0, image, ph) {
            console::println("elf: segment map failed");
            return None;
        }
    }

    Some(LoadedElf {
        entry: ehdr.e_entry as usize,
    })
}

fn load_segment(ttbr0: usize, image: &[u8], ph: &Phdr) -> bool {
    let vaddr = ph.p_vaddr as usize;
    let memsz = ph.p_memsz as usize;
    let filesz = ph.p_filesz as usize;
    let file_off = ph.p_offset as usize;
    if file_off + filesz > image.len() {
        return false;
    }

    let kind = if (ph.p_flags & PF_X) != 0 {
        UserMap::Text
    } else {
        UserMap::Data
    };
    // Writable segments must be Data even if also executable (rare for us).
    let kind = if (ph.p_flags & PF_W) != 0 {
        UserMap::Data
    } else {
        kind
    };

    let start = vaddr & !(PAGE_SIZE - 1);
    let end = (vaddr + memsz + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
    let mut page = start;
    while page < end {
        let phys = match frame::alloc_frame() {
            Some(p) => p,
            None => return false,
        };
        // Copy file bytes that overlap this page.
        let page_end = page + PAGE_SIZE;
        let seg_start = vaddr;
        let seg_file_end = vaddr + filesz;
        let seg_mem_end = vaddr + memsz;

        let copy_start = core::cmp::max(page, seg_start);
        let copy_end = core::cmp::min(page_end, seg_file_end);
        if copy_start < copy_end {
            let dst_off = copy_start - page;
            let src_off = file_off + (copy_start - seg_start);
            let len = copy_end - copy_start;
            unsafe {
                core::ptr::copy_nonoverlapping(
                    image.as_ptr().add(src_off),
                    (phys + dst_off) as *mut u8,
                    len,
                );
            }
        }
        // BSS region already zeroed by alloc_frame.
        let _ = seg_mem_end;

        if !vm::map_user_page(ttbr0, page, phys, kind) {
            return false;
        }
        page += PAGE_SIZE;
    }
    true
}
