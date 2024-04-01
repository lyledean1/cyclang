; ModuleID = 'builtins-wasm32'
source_filename = "builtins-wasm32"
target datalayout = "e-m:e-p:32:32-p10:8:8-p20:8:8-i64:64-n32:64-S128-ni:1:10:20"
target triple = "wasm32-unknown-unknown-unknown"

%target.Target.Cpu.Feature.Set = type { [9 x i32] }
%target.Target.Cpu.Model = type { { ptr, i32 }, { ptr, i32 }, %target.Target.Cpu.Feature.Set }
%target.Target.Cpu = type { ptr, %target.Target.Cpu.Feature.Set, i6, [3 x i8] }

@builtin.zig_backend = internal unnamed_addr constant i64 2, align 8
@target.Target.Cpu.Feature.Set.empty = internal unnamed_addr constant %target.Target.Cpu.Feature.Set zeroinitializer, align 4
@target.wasm.cpu.generic = internal unnamed_addr constant %target.Target.Cpu.Model { { ptr, i32 } { ptr @target.wasm.cpu.generic__anon_421, i32 7 }, { ptr, i32 } { ptr @target.wasm.cpu.generic__anon_421, i32 7 }, %target.Target.Cpu.Feature.Set { [9 x i32] [i32 544, i32 0, i32 0, i32 0, i32 0, i32 0, i32 0, i32 0, i32 0] } }, align 4
@target.wasm.cpu.generic__anon_421 = internal unnamed_addr constant [8 x i8] c"generic\00", align 1
@builtin.cpu = internal unnamed_addr constant %target.Target.Cpu { ptr @target.wasm.cpu.generic, %target.Target.Cpu.Feature.Set { [9 x i32] [i32 544, i32 0, i32 0, i32 0, i32 0, i32 0, i32 0, i32 0, i32 0] }, i6 -9, [3 x i8] undef }, align 4
@start.simplified_logic = internal unnamed_addr constant i1 false, align 1
@builtin.output_mode = internal unnamed_addr constant i2 -2, align 1
@builtin.panic_messages.integer_overflow__anon_802 = internal unnamed_addr constant [17 x i8] c"integer overflow\00", align 1
@0 = private unnamed_addr constant { i32, i8, [3 x i8] } { i32 undef, i8 0, [3 x i8] undef }, align 4
@builtin.panic_messages.integer_overflow = internal unnamed_addr constant ptr @builtin.panic_messages.integer_overflow__anon_802, align 4
@builtin.os = internal unnamed_addr global { { [84 x i8], i2, [3 x i8] }, i6, [3 x i8] } { { [84 x i8], i2, [3 x i8] } { [84 x i8] undef, i2 0, [3 x i8] undef }, i6 0, [3 x i8] undef }, align 4

; Function Attrs: noredzone nounwind
define dso_local i32 @add(i32 %0, i32 %1) #0 {
  %3 = call fastcc { i32, i1 } @llvm.sadd.with.overflow.i32(i32 %0, i32 %1)
  %4 = extractvalue { i32, i1 } %3, 1
  br i1 %4, label %5, label %6

5:                                                ; preds = %2
  call fastcc void @builtin.default_panic(ptr @builtin.panic_messages.integer_overflow__anon_802, i32 16, ptr null, ptr @0)
  unreachable

6:                                                ; preds = %2
  %7 = extractvalue { i32, i1 } %3, 0
  ret i32 %7
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare { i32, i1 } @llvm.sadd.with.overflow.i32(i32, i32) #1

; Function Attrs: cold noredzone noreturn nounwind
define internal fastcc void @builtin.default_panic(ptr nonnull readonly align 1 %0, i32 %1, ptr align 4 %2, ptr nonnull readonly align 4 %3) unnamed_addr #2 {
  %5 = insertvalue { ptr, i32 } poison, ptr %0, 0
  %6 = insertvalue { ptr, i32 } %5, i32 %1, 1
  br label %7

7:                                                ; preds = %7, %4
  call void @llvm.debugtrap()
  br label %7
}

; Function Attrs: nounwind
declare void @llvm.debugtrap() #3

attributes #0 = { noredzone nounwind "frame-pointer"="all" "target-features"="+mutable-globals,+sign-ext," }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) "target-features"="+mutable-globals,+sign-ext," }
attributes #2 = { cold noredzone noreturn nounwind "frame-pointer"="all" "target-features"="+mutable-globals,+sign-ext," }
attributes #3 = { nounwind "target-features"="+mutable-globals,+sign-ext," }

!llvm.module.flags = !{!0, !1}

!0 = !{i32 1, !"wasm-feature-mutable-globals", i32 43}
!1 = !{i32 1, !"wasm-feature-sign-ext", i32 43}
