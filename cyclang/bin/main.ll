; ModuleID = 'main'
source_filename = "main"
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"

@true_str = private unnamed_addr constant [6 x i8] c"true\0A\00", align 1
@false_str = private unnamed_addr constant [7 x i8] c"false\0A\00", align 1
@number_printf_val = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@str_printf_val = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1

define void @main() {
main:
  %num = alloca ptr, align 8
  store i32 2, ptr %num, align 4
  %num1 = alloca ptr, align 8
  store i32 4, ptr %num1, align 4
  %0 = load i32, ptr %num, align 4
  %"\F0\A402\01" = load i32, ptr %num1, align 4
  %addNumberType = add i32 %0, %"\F0\A402\01"
  store i32 %addNumberType, ptr %num, align 4
  %"\00\00\00\00\00" = load i32, ptr %num, align 4
  call void (ptr, ...) @printf(ptr @number_printf_val, i32 %"\00\00\00\00\00")
  ret void
}

define ptr @bool_to_str(i1 %0) {
entry:
  %true_str.false_str = select i1 %0, ptr @true_str, ptr @false_str
  ret ptr %true_str.false_str
}

declare void @printf(ptr, ...)

declare ptr @sprintf(ptr, ptr, ptr, ptr, ...)
