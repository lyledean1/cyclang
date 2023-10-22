; ModuleID = 'main'
source_filename = "main"
target triple = "arm64"

@true_str = private unnamed_addr constant [6 x i8] c"true\0A\00", align 1
@false_str = private unnamed_addr constant [7 x i8] c"false\0A\00", align 1
@number_printf_val = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@str_printf_val = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1

define void @main() {
main:
  %0 = call i64 @fib(i64 45)
  call void (ptr, ...) @printf(ptr @number_printf_val, i64 %0)
  ret void
}

declare void @printf(ptr, ...)

declare ptr @sprintf(ptr, ptr, ptr, ptr, ...)

define i64 @fib(i64 %0) {
entry:
  %cmp = icmp sle i64 %0, 1
  br i1 %cmp, label %then_block, label %else_block

then_block:                                       ; preds = %entry
  %is_zero = icmp eq i64 %0, 0
  br i1 %is_zero, label %return_zero, label %return_one

return_zero:                                      ; preds = %then_block
  ret i64 0

return_one:                                       ; preds = %then_block
  ret i64 1

else_block:                                       ; preds = %entry
  %x = sub i64 %0, 1
  %1 = call i64 @fib(i64 %x)
  %y = sub i64 %0, 2
  %2 = call i64 @fib(i64 %y)
  %add_num = add i64 %1, %2
  ret i64 %add_num
}
