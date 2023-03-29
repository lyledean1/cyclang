	.text
	.file	"main"
	.globl	main                            // -- Begin function main
	.p2align	2
	.type	main,@function
main:                                   // @main
	.cfi_startproc
// %bb.0:                               // %main
	sub	sp, sp, #384                    // =384
	stp	x29, x30, [sp, #368]            // 16-byte Folded Spill
	.cfi_def_cfa_offset 384
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	adrp	x0, .L__unnamed_1
	mov	w8, #5
	add	x0, x0, :lo12:.L__unnamed_1
	mov	w1, #5
	str	w8, [sp, #364]
	bl	printf
	adrp	x0, .L__unnamed_2
	mov	w8, #24
	add	x0, x0, :lo12:.L__unnamed_2
	mov	w1, #24
	str	w8, [sp, #360]
	bl	printf
	adrp	x0, .L__unnamed_3
	mov	w8, #16
	add	x0, x0, :lo12:.L__unnamed_3
	mov	w1, #16
	str	w8, [sp, #356]
	bl	printf
	adrp	x0, .L__unnamed_4
	mov	w8, #20
	add	x0, x0, :lo12:.L__unnamed_4
	mov	w1, #20
	str	w8, [sp, #352]
	bl	printf
	adrp	x0, .L__unnamed_5
	adrp	x1, .Ltrue_str
	add	x0, x0, :lo12:.L__unnamed_5
	add	x1, x1, :lo12:.Ltrue_str
	bl	printf
	adrp	x1, .Lformat_str
	mov	w8, #34
	mov	w9, #100
	mov	w10, #108
	mov	w11, #114
	mov	w12, #111
	mov	w13, #119
	mov	w14, #32
	add	x1, x1, :lo12:.Lformat_str
	add	x0, sp, #96                     // =96
	mov	w2, #34
	mov	w3, #104
	mov	w4, #101
	mov	w5, #108
	mov	w6, #108
	mov	w7, #111
	str	wzr, [sp, #80]
	str	w9, [sp, #64]
	str	w8, [sp, #72]
	str	w10, [sp, #56]
	str	w11, [sp, #48]
	str	w12, [sp, #40]
	str	w13, [sp, #32]
	str	w14, [sp, #24]
	str	w8, [sp, #16]
	str	w8, [sp]
	str	wzr, [sp, #8]
	bl	sprintf
	adrp	x0, .L__unnamed_6
	adrp	x1, ".LHE–"
	add	x0, x0, :lo12:.L__unnamed_6
	add	x1, x1, :lo12:".LHE–"
	bl	printf
	ldp	x29, x30, [sp, #368]            // 16-byte Folded Reload
	add	sp, sp, #384                    // =384
	ret
.Lfunc_end0:
	.size	main, .Lfunc_end0-main
	.cfi_endproc
                                        // -- End function
	.type	.L__unnamed_1,@object           // @0
	.section	.rodata.str1.1,"aMS",@progbits,1
.L__unnamed_1:
	.asciz	"%d\n"
	.size	.L__unnamed_1, 4

	.type	.L__unnamed_2,@object           // @1
.L__unnamed_2:
	.asciz	"%d\n"
	.size	.L__unnamed_2, 4

	.type	.L__unnamed_3,@object           // @2
.L__unnamed_3:
	.asciz	"%d\n"
	.size	.L__unnamed_3, 4

	.type	.L__unnamed_4,@object           // @3
.L__unnamed_4:
	.asciz	"%d\n"
	.size	.L__unnamed_4, 4

	.type	.Ltrue_str,@object              // @true_str
.Ltrue_str:
	.asciz	"true"
	.size	.Ltrue_str, 5

	.type	.L__unnamed_5,@object           // @4
.L__unnamed_5:
	.asciz	"%s\n"
	.size	.L__unnamed_5, 4

	.type	.Lformat_str,@object            // @format_str
	.section	.rodata.str1.4,"aMS",@progbits,1
	.p2align	2
.Lformat_str:
	.asciz	"%s%s"
	.size	.Lformat_str, 5

	.type	.L__unnamed_6,@object           // @5
	.section	.rodata.str1.1,"aMS",@progbits,1
.L__unnamed_6:
	.asciz	"%s\n"
	.size	.L__unnamed_6, 4

	.type	".LHE–",@object                  // @"HE\96\02"
".LHE–":
	.asciz	"HE\226\002"
	.size	".LHE–", 5

	.section	".note.GNU-stack","",@progbits
