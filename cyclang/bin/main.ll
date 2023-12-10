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
  store i32 1, ptr %num, align 4
  %num1 = alloca ptr, align 8
  store i32 2, ptr %num1, align 4
  %num2 = alloca ptr, align 8
  store i32 3, ptr %num2, align 4
  %num3 = alloca ptr, align 8
  store i32 4, ptr %num3, align 4
  %my_array = alloca [4 x i32], align 4
  store [4 x i32] [i32 1, i32 2, i32 3, i32 4], ptr %my_array, align 4
  %num4 = alloca ptr, align 8
  store i32 0, ptr %num4, align 4
  %access_array = getelementptr [4 x i32], ptr %my_array, i32 0, i32 0
  %access_array5 = load i32, ptr %access_array, align 4
  call void (ptr, ...) @printf(ptr @number_printf_val, i32 %access_array5)
  %num6 = alloca ptr, align 8
  store i32 1, ptr %num6, align 4
  %access_array7 = getelementptr [4 x i32], ptr %my_array, i32 0, i32 1
  %access_array8 = load i32, ptr %access_array7, align 4
  call void (ptr, ...) @printf(ptr @number_printf_val, i32 %access_array8)
  %num9 = alloca ptr, align 8
  store i32 2, ptr %num9, align 4
  %access_array10 = getelementptr [4 x i32], ptr %my_array, i32 0, i32 2
  %access_array11 = load i32, ptr %access_array10, align 4
  call void (ptr, ...) @printf(ptr @number_printf_val, i32 %access_array11)
  %num12 = alloca ptr, align 8
  store i32 3, ptr %num12, align 4
  %access_array13 = getelementptr [4 x i32], ptr %my_array, i32 0, i32 3
  %access_array14 = load i32, ptr %access_array13, align 4
  call void (ptr, ...) @printf(ptr @number_printf_val, i32 %access_array14)
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
