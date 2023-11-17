; ModuleID = './bin/main.ll'
source_filename = "main"
target triple = "wasm32-unknown-unknown-wasm"

@true_str = private unnamed_addr constant [6 x i8] c"true\0A\00", align 1
@false_str = private unnamed_addr constant [7 x i8] c"false\0A\00", align 1

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none)
define nonnull ptr @bool_to_str(i1 %0) local_unnamed_addr #0 {
entry:
  %true_str.false_str = select i1 %0, ptr @true_str, ptr @false_str
  ret ptr %true_str.false_str
}

; Function Attrs: nofree nosync nounwind memory(none)
define i32 @fib(i32 %0) local_unnamed_addr #1 {
entry:
  %result1 = icmp slt i32 %0, 2
  br i1 %result1, label %common.ret, label %else_block

common.ret:                                       ; preds = %else_block, %entry
  %accumulator.tr.lcssa = phi i32 [ 0, %entry ], [ %add, %else_block ]
  %.tr.lcssa = phi i32 [ %0, %entry ], [ %sub3, %else_block ]
  %accumulator.ret.tr = add i32 %.tr.lcssa, %accumulator.tr.lcssa
  ret i32 %accumulator.ret.tr

else_block:                                       ; preds = %entry, %else_block
  %.tr3 = phi i32 [ %sub3, %else_block ], [ %0, %entry ]
  %accumulator.tr2 = phi i32 [ %add, %else_block ], [ 0, %entry ]
  %sub = add nsw i32 %.tr3, -1
  %1 = tail call i32 @fib(i32 %sub)
  %sub3 = add nsw i32 %.tr3, -2
  %add = add i32 %1, %accumulator.tr2
  %result = icmp ult i32 %.tr3, 4
  br i1 %result, label %common.ret, label %else_block
}

attributes #0 = { mustprogress nofree norecurse nosync nounwind willreturn memory(none) }
attributes #1 = { nofree nosync nounwind memory(none) }
