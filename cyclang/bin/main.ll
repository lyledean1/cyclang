; ModuleID = 'main'
source_filename = "main"

@number_printf_val = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@number64_printf_val = private unnamed_addr constant [6 x i8] c"%llu\0A\00", align 1
@str_printf_val = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1
@true_str = private unnamed_addr constant [6 x i8] c"true\0A\00", align 1
@false_str = private unnamed_addr constant [7 x i8] c"false\0A\00", align 1

define void @main() {
main:
  %bool_value = alloca i1, align 1
  store i1 true, ptr %bool_value, align 1
  %0 = load i1, ptr %bool_value, align 1
  %1 = call ptr @bool_to_str(i1 %0)
  call void (ptr, ...) @printf(ptr %1)
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
