; ModuleID = 'main'
source_filename = "main"
target triple = "arm64"

@0 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@1 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@2 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@3 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@true_str = private unnamed_addr constant [5 x i8] c"true\00", align 1
@4 = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1
@5 = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1
@"hello world this is more data" = private unnamed_addr constant [30 x i8] c"hello world this is more data\00", align 1

define void @main() {
main:
  %value = alloca i32, align 4
  store i32 fdiv (i32 20, i32 4), i32* %value, align 4
  %value1 = load i32, i32* %value, align 4
  call void (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @0, i32 0, i32 0), i32 %value1)
  %value2 = alloca i32, align 4
  store i32 24, i32* %value2, align 4
  %value3 = load i32, i32* %value2, align 4
  call void (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @1, i32 0, i32 0), i32 %value3)
  %value4 = alloca i32, align 4
  store i32 16, i32* %value4, align 4
  %value5 = load i32, i32* %value4, align 4
  call void (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @2, i32 0, i32 0), i32 %value5)
  %value6 = alloca i32, align 4
  store i32 20, i32* %value6, align 4
  %value7 = load i32, i32* %value6, align 4
  call void (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @3, i32 0, i32 0), i32 %value7)
  call void (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @4, i32 0, i32 0), i8* getelementptr inbounds ([5 x i8], [5 x i8]* @true_str, i32 0, i32 0))
  call void (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @5, i32 0, i32 0), i8* getelementptr inbounds ([30 x i8], [30 x i8]* @"hello world this is more data", i32 0, i32 0))
  ret void
}

declare void @printf(i8*, ...)

declare i8* @sprintf(i8*, i8*, i8*, i8*, ...)
