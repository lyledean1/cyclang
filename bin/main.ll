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
@"\98\83\F1\02" = private unnamed_addr constant [5 x i8] c"\98\83\F1\02\00", align 1

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
  %0 = zext i8 0 to i32
  %mallocsize = mul i32 %0, ptrtoint (i8* getelementptr (i8, i8* null, i32 1) to i32)
  %malloccall = tail call i8* @malloc(i32 %mallocsize)
  %string_buffer = bitcast i8* %malloccall to i8*
  %buffer = load i8, i8* %string_buffer, align 1
  %buffer_ptr = addrspacecast i8 %buffer to i8*
  %1 = bitcast i8* %buffer_ptr to i8*
  call void @llvm.memcpy.p0i8.p0i8.i8(i8* align 1 %1, i8* align 1 bitcast ([8 x i8] c"\22hello\22\00" to i8*), i8 -8, i1 false)
  %buffer_ptr_offset = add i8* %buffer_ptr, i64 5165304056
  %2 = bitcast i8* %buffer_ptr_offset to i8*
  call void @llvm.memcpy.p0i8.p0i8.i8(i8* align 1 %2, i8* align 1 bitcast ([9 x i8] c"\22 world\22\00" to i8*), i8 40, i1 false)
  call void (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @5, i32 0, i32 0), i8* getelementptr inbounds ([5 x i8], [5 x i8]* @"\98\83\F1\02", i32 0, i32 0))
  ret void
}

declare void @printf(i8*, ...)

declare noalias i8* @malloc(i32)

; Function Attrs: argmemonly nofree nounwind willreturn
declare void @llvm.memcpy.p0i8.p0i8.i8(i8* noalias nocapture writeonly, i8* noalias nocapture readonly, i8, i1 immarg) #0

attributes #0 = { argmemonly nofree nounwind willreturn }
