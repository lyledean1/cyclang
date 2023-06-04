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
  %bool_var = alloca i1, align 1
  store i1 true, i1* %bool_var
  br label %loop_cond

loop_cond:                                         ; preds = %loop_body, %entry
  %b = load i1, i1* %bool_var
  br i1 %b, label %loop_body, label %loop_exit

loop_body:                                        ; preds = %loop_cond
  call void (ptr, ...) @printf(ptr @0, ptr @"\22here\22")
  store i1 false, i1* %bool_var
  br label %loop_cond

loop_exit:                                        ; preds = %loop_cond
  ret void
}