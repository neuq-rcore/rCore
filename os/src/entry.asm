.section .text.entry

	.global _start
_start:
	la sp, boot_stack_top
	# Make fp 0 so that stack trace knows where to stop
	xor fp, fp, fp
	j __kernel_start_main

.section .bss.stack

	.global boot_stack_lower_bound
boot_stack_lower_bound:
	.space 0x00080000 # 512 KB

	.global boot_stack_top
boot_stack_top:
