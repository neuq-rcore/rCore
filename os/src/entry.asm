.section .bss.stack

	.global boot_stack_lower_bound
boot_stack_lower_bound:
	.space 0x00080000 # 512 KB

	.global boot_stack_top
boot_stack_top:
