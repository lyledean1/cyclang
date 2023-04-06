; ModuleID = 'main'
source_filename = "main"
target triple = "arm64"

@0 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1

define void @main() {
main:
  %value = alloca i32, align 4
  store i32 10, i32* %value, align 4
  %value1 = load i32, i32* %value, align 4
  call void (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @0, i32 0, i32 0), i32 %value1)
  ret void
}

declare void @printf(i8*, ...)

declare i8* @sprintf(i8*, i8*, i8*, i8*, ...)
