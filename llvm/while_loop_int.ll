; ModuleID = 'main'
source_filename = "main"
target triple = "arm64"

@0 = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1
@"\22here\22" = private unnamed_addr constant [7 x i8] c"\22here\22\00", align 1
@true_str = private unnamed_addr constant [5 x i8] c"true\00", align 1
@false_str = private unnamed_addr constant [6 x i8] c"false\00", align 1
@1 = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1

define void @main() {
main:
  %value_bool_var = alloca i1, align 1
  store i1 true, ptr %value_bool_var, align 1
  call void @while_loop()
  ret void
}

declare void @printf(ptr, ...)

declare ptr @sprintf(ptr, ptr, ptr, ptr, ...)

define void @while_loop() {
entry:
  %counter = alloca i32, align 4
  store i32 0, i32* %counter
  br label %loop_cond

loop_cond:                                         ; preds = %loop_body, %entry
  %c = load i32, i32* %counter
  %cond = icmp slt i32 %c, 5
  br i1 %cond, label %loop_body, label %loop_exit

loop_body:                                        ; preds = %loop_cond
  %0 = load i32, i32* %counter
  call void (ptr, ...) @printf(ptr @0, ptr @"\22here\22")
  %next = add i32 %0, 1
  store i32 %next, i32* %counter
  br label %loop_cond

loop_exit:                                        ; preds = %loop_cond
  ret void
}