; ModuleID = 'main'
source_filename = "main"

@true_str = private unnamed_addr constant [6 x i8] c"true\0A\00", align 1
@false_str = private unnamed_addr constant [7 x i8] c"false\0A\00", align 1
@number_printf_val = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@number64_printf_val = private unnamed_addr constant [6 x i8] c"%llu\0A\00", align 1
@str_printf_val = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1

define void @main() {
main:
  %num = alloca ptr, align 8
  store i32 20, ptr %num, align 4
  %call_arg = load i32, ptr %num, align 4
  %0 = call i32 @fib(i32 %call_arg)
  %call_value_int32 = alloca ptr, align 8
  store i32 %0, ptr %call_value_int32, align 4
  call void (ptr, ...) @printf(ptr @number_printf_val, i32 %0)
  ret void
}

define ptr @bool_to_str(i1 %0) {
entry:
  br i1 %0, label %then, label %else

then:                                             ; preds = %entry
  ret ptr @true_str

else:                                             ; preds = %entry
  ret ptr @false_str
}

declare void @printf(ptr, ...)

declare ptr @sprintf(ptr, ptr, ptr, ptr, ...)

define i32 @fib(i32 %0) {
entry:
  %num = alloca ptr, align 8
  store i32 2, ptr %num, align 4
  %result = icmp slt i32 %0, 2
  %bool_cmp = alloca i1, align 1
  store i1 %result, ptr %bool_cmp, align 1
  %cmp = load i1, ptr %bool_cmp, align 1
  br i1 %cmp, label %then_block, label %else_block

then_block:                                       ; preds = %entry
  ret i32 %0

merge_block:                                      ; preds = %else_block
  %num1 = alloca ptr, align 8
  store i32 1, ptr %num1, align 4
  %sub = sub i32 %0, 1
  %param_add = alloca ptr, align 8
  store i32 %sub, ptr %param_add, align 4
  %call_arg = load i32, ptr %param_add, align 4
  %1 = call i32 @fib(i32 %call_arg)
  %call_value_int32 = alloca ptr, align 8
  store i32 %1, ptr %call_value_int32, align 4
  %num2 = alloca ptr, align 8
  store i32 2, ptr %num2, align 4
  %sub3 = sub i32 %0, 2
  %param_add4 = alloca ptr, align 8
  store i32 %sub3, ptr %param_add4, align 4
  %call_arg5 = load i32, ptr %param_add4, align 4
  %2 = call i32 @fib(i32 %call_arg5)
  %call_value_int326 = alloca ptr, align 8
  store i32 %2, ptr %call_value_int326, align 4
  %add = add i32 %1, %2
  %param_add7 = alloca ptr, align 8
  store i32 %add, ptr %param_add7, align 4
  ret i32 %add

else_block:                                       ; preds = %entry
  br label %merge_block
}
