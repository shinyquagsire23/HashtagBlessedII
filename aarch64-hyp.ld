OUTPUT_ARCH(aarch64)
ENTRY(_start)

MEMORY
{
  ram : ORIGIN = 0xD0000000, LENGTH = 0x4000000
}

SECTIONS
{
    . = ALIGN(0x1000);
    .text :   /* ALIGN (4): */
    {
        __text_start = . ;
        KEEP (*(.crt0))
        KEEP (*(.init))
        *(.text .stub .text.* .gnu.linkonce.t.*)
        KEEP (*(.text.*personality*))
        /* .gnu.warning sections are handled specially by elf32.em.  */
        *(.gnu.warning)
        *(.glue_7t) *(.glue_7) *(.vfp11_veneer)
        *(.dynbss)
        *(.gnu.linkonce.b*)
        . = ALIGN(4);  /* REQUIRED. LD is flaky without it. */
        
        *(.rodata)
        *all.rodata*(*)
        *(.roda)
        *(.rodata.*)
        *(.gnu.linkonce.r*)
        *(.eh_frame)
        *(.eh_frame_hdr)
        SORT(CONSTRUCTORS)
        . = ALIGN(4);   /* REQUIRED. LD is flaky without it. */
    } >ram = 0xff

    . = ALIGN(0x1000);
    __text_end = . ;
    __data_start = . ;
    
    .data ALIGN(16) :    {
        
        *(.data)
        *(.data.*)
        
        /* Ensure the __preinit_array_start label is properly aligned.  We
		   could instead move the label definition inside the section, but
		   the linker would then create the section even if it turns out to
		   be empty, which isn't pretty.  */
		. = ALIGN(32 / 8);
		PROVIDE (__preinit_array_start = .);
		KEEP (*(.preinit_array))
		PROVIDE (__preinit_array_end = .);
		PROVIDE (__init_array_start = .);
		KEEP (*(.init_array))
		PROVIDE (__init_array_end = .);
		PROVIDE (__fini_array_start = .);
		KEEP (*(.fini_array))
		PROVIDE (__fini_array_end = .);
        
        /* gcc uses crtbegin.o to find the start of the constructors, so
        we make sure it is first.  Because this is a wildcard, it
        doesn't matter if the user does not actually link against
        crtbegin.o; the linker won't look for a file to match a
        wildcard.  The wildcard also means that it doesn't matter which
        directory crtbegin.o is in.  */
        KEEP (*crtbegin.o(.ctors))
        KEEP (*(EXCLUDE_FILE (*crtend.o) .ctors))
        KEEP (*(SORT(.ctors.*)))
        KEEP (*(.ctors))
        . = ALIGN(4);   /* REQUIRED. LD is flaky without it. */
        
        KEEP (*crtbegin.o(.dtors))
        KEEP (*(EXCLUDE_FILE (*crtend.o) .dtors))
        KEEP (*(SORT(.dtors.*)))
        KEEP (*(.dtors))
        . = ALIGN(4);   /* REQUIRED. LD is flaky without it. */
        
        KEEP (*(.eh_frame))
        . = ALIGN(4);   /* REQUIRED. LD is flaky without it. */
        
        *(.gcc_except_table)
        . = ALIGN(4);   /* REQUIRED. LD is flaky without it. */
        
        KEEP (*(.jcr))
        *(.got.plt) *(.got)
        *(.ARM.extab* .gnu.linkonce.armextab.*)
        *(.ARM.exidx* .gnu.linkonce.armexidx.*)
        
        *(.plt)
        *(.gnu.linkonce.d*)
        KEEP (*(.fini))
        
        *(COMMON)
        CONSTRUCTORS
        
        . = ALIGN(4);    /* REQUIRED. LD is flaky without it. */
    } >ram = 0xff

    . = ALIGN(0x1000);
    
    .stack ALIGN(4) : {
        _stack = .; *(.stack)
        . = ALIGN(0x100);
        __stack_start = .;
        . += 0x80000;
        __stack_end = .;
        LD_STACK_PTR = .;
        
    } >ram = 0xff
    
    . = ALIGN(0x1000);
    
    .bss ALIGN(4) : {
        __bss_start = .;
    	*(.bss*)
        *(.bss)
        *(.bss.*)
        . = ALIGN(0x20);
        __bss_end = .;
        
        . = ALIGN(0x100000);
    } >ram = 0xff
    
    

    __data_end = . ;
    __end__ = .;

    /* Stabs debugging sections.  */
    .stab 0 : { *(.stab) }
    .stabstr 0 : { *(.stabstr) }
    .stab.excl 0 : { *(.stab.excl) }
    .stab.exclstr 0 : { *(.stab.exclstr) }
    .stab.index 0 : { *(.stab.index) }
    .stab.indexstr 0 : { *(.stab.indexstr) }
    .comment 0 : { *(.comment) }
    /*  DWARF debug sections.
        Symbols in the DWARF debugging sections are relative to the beginning
        of the section so we begin them at 0.  */
    /* DWARF 1 */
    .debug          0 : { *(.debug) }
    .line           0 : { *(.line) }
    /* GNU DWARF 1 extensions */
    .debug_srcinfo  0 : { *(.debug_srcinfo) }
    .debug_sfnames  0 : { *(.debug_sfnames) }
    /* DWARF 1.1 and DWARF 2 */
    .debug_aranges  0 : { *(.debug_aranges) }
    .debug_pubnames 0 : { *(.debug_pubnames) }
    /* DWARF 2 */
    .debug_info     0 : { *(.debug_info) }
    .debug_abbrev   0 : { *(.debug_abbrev) }
    .debug_line     0 : { *(.debug_line) }
    .debug_frame    0 : { *(.debug_frame) }
    .debug_str      0 : { *(.debug_str) }
    .debug_loc      0 : { *(.debug_loc) }
    .debug_macinfo  0 : { *(.debug_macinfo) }
    /* SGI/MIPS DWARF 2 extensions */
    .debug_weaknames 0 : { *(.debug_weaknames) }
    .debug_funcnames 0 : { *(.debug_funcnames) }
    .debug_typenames 0 : { *(.debug_typenames) }
    .debug_varnames  0 : { *(.debug_varnames) }
    
    .defmt 0 : {  }
    
    /* These must appear regardless of  .  */
}
