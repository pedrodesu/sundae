; ModuleID = 'b.ce150141dbfb6a70-cgu.0'
source_filename = "b.ce150141dbfb6a70-cgu.0"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

%"core::fmt::Arguments<'_>" = type { { ptr, i64 }, { ptr, i64 }, { ptr, i64 } }

@vtable.0 = private unnamed_addr constant <{ ptr, [16 x i8], ptr, ptr, ptr }> <{ ptr @"_ZN4core3ptr85drop_in_place$LT$std..rt..lang_start$LT$$LP$$RP$$GT$..$u7b$$u7b$closure$u7d$$u7d$$GT$17haadc6c59042cae13E", [16 x i8] c"\08\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", ptr @"_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17hc1ec4a51ea3678a9E", ptr @"_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h0ca664f915dfe515E", ptr @"_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h0ca664f915dfe515E" }>, align 8
@alloc_a7129c0d482e8ad6e048a7287538fe3c = private unnamed_addr constant <{ [13 x i8] }> <{ [13 x i8] c"Hello world!\0A" }>, align 1
@alloc_30680aa952a20333f4e5d0ccb639e34a = private unnamed_addr constant <{ ptr, [8 x i8] }> <{ ptr @alloc_a7129c0d482e8ad6e048a7287538fe3c, [8 x i8] c"\0D\00\00\00\00\00\00\00" }>, align 8
@alloc_513570631223a12912d85da2bec3b15a = private unnamed_addr constant <{}> zeroinitializer, align 8

; std::sys_common::backtrace::__rust_begin_short_backtrace
; Function Attrs: noinline nonlazybind uwtable
define internal fastcc void @_ZN3std10sys_common9backtrace28__rust_begin_short_backtrace17h85e14443d54d0859E(ptr nocapture noundef nonnull readonly %f) unnamed_addr #0 {
start:
  tail call void %f()
  tail call void asm sideeffect "", "~{memory}"() #6, !srcloc !4
  ret void
}

; std::rt::lang_start
; Function Attrs: nonlazybind uwtable
define hidden noundef i64 @_ZN3std2rt10lang_start17h8924776ceb23b21dE(ptr noundef nonnull %main, i64 noundef %argc, ptr noundef %argv, i8 noundef %sigpipe) unnamed_addr #1 {
start:
  %_8 = alloca ptr, align 8
  call void @llvm.lifetime.start.p0(i64 8, ptr nonnull %_8)
  store ptr %main, ptr %_8, align 8
; call std::rt::lang_start_internal
  %0 = call noundef i64 @_ZN3std2rt19lang_start_internal17h56699df87ce50e6fE(ptr noundef nonnull align 1 %_8, ptr noalias noundef nonnull readonly align 8 dereferenceable(24) @vtable.0, i64 noundef %argc, ptr noundef %argv, i8 noundef %sigpipe)
  call void @llvm.lifetime.end.p0(i64 8, ptr nonnull %_8)
  ret i64 %0
}

; std::rt::lang_start::{{closure}}
; Function Attrs: inlinehint nonlazybind uwtable
define internal noundef i32 @"_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h0ca664f915dfe515E"(ptr noalias nocapture noundef readonly align 8 dereferenceable(8) %_1) unnamed_addr #2 {
start:
  %_4 = load ptr, ptr %_1, align 8, !nonnull !5, !noundef !5
; call std::sys_common::backtrace::__rust_begin_short_backtrace
  tail call fastcc void @_ZN3std10sys_common9backtrace28__rust_begin_short_backtrace17h85e14443d54d0859E(ptr noundef nonnull %_4)
  ret i32 0
}

; core::ops::function::FnOnce::call_once{{vtable.shim}}
; Function Attrs: inlinehint nonlazybind uwtable
define internal noundef i32 @"_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17hc1ec4a51ea3678a9E"(ptr nocapture noundef readonly %_1) unnamed_addr #2 personality ptr @rust_eh_personality {
start:
  %0 = load ptr, ptr %_1, align 8, !nonnull !5, !noundef !5
; call std::sys_common::backtrace::__rust_begin_short_backtrace
  tail call fastcc void @_ZN3std10sys_common9backtrace28__rust_begin_short_backtrace17h85e14443d54d0859E(ptr noundef nonnull %0), !noalias !6
  ret i32 0
}

; core::ptr::drop_in_place<std::rt::lang_start<()>::{{closure}}>
; Function Attrs: inlinehint mustprogress nofree norecurse nosync nounwind nonlazybind willreturn memory(none) uwtable
define internal void @"_ZN4core3ptr85drop_in_place$LT$std..rt..lang_start$LT$$LP$$RP$$GT$..$u7b$$u7b$closure$u7d$$u7d$$GT$17haadc6c59042cae13E"(ptr noalias nocapture readnone align 8 %_1) unnamed_addr #3 {
start:
  ret void
}

; b::main
; Function Attrs: nonlazybind uwtable
define internal void @_ZN1b4main17he022c5120c0a591eE() unnamed_addr #1 {
start:
  %_2 = alloca %"core::fmt::Arguments<'_>", align 8
  call void @llvm.lifetime.start.p0(i64 48, ptr nonnull %_2)
  store ptr @alloc_30680aa952a20333f4e5d0ccb639e34a, ptr %_2, align 8
  %0 = getelementptr inbounds { ptr, i64 }, ptr %_2, i64 0, i32 1
  store i64 1, ptr %0, align 8
  %1 = getelementptr inbounds %"core::fmt::Arguments<'_>", ptr %_2, i64 0, i32 2
  store ptr null, ptr %1, align 8
  %2 = getelementptr inbounds %"core::fmt::Arguments<'_>", ptr %_2, i64 0, i32 1
  store ptr @alloc_513570631223a12912d85da2bec3b15a, ptr %2, align 8
  %3 = getelementptr inbounds %"core::fmt::Arguments<'_>", ptr %_2, i64 0, i32 1, i32 1
  store i64 0, ptr %3, align 8
; call std::io::stdio::_print
  call void @_ZN3std2io5stdio6_print17h1a5e9ff8095f267bE(ptr noalias nocapture noundef nonnull align 8 dereferenceable(48) %_2)
  call void @llvm.lifetime.end.p0(i64 48, ptr nonnull %_2)
  ret void
}

; std::rt::lang_start_internal
; Function Attrs: nonlazybind uwtable
declare noundef i64 @_ZN3std2rt19lang_start_internal17h56699df87ce50e6fE(ptr noundef nonnull align 1, ptr noalias noundef readonly align 8 dereferenceable(24), i64 noundef, ptr noundef, i8 noundef) unnamed_addr #1

; Function Attrs: nonlazybind uwtable
declare noundef i32 @rust_eh_personality(i32 noundef, i32 noundef, i64 noundef, ptr noundef, ptr noundef) unnamed_addr #1

; std::io::stdio::_print
; Function Attrs: nonlazybind uwtable
declare void @_ZN3std2io5stdio6_print17h1a5e9ff8095f267bE(ptr noalias nocapture noundef align 8 dereferenceable(48)) unnamed_addr #1

; Function Attrs: nonlazybind
define i32 @main(i32 %0, ptr %1) unnamed_addr #4 {
top:
  %_8.i = alloca ptr, align 8
  %2 = sext i32 %0 to i64
  call void @llvm.lifetime.start.p0(i64 8, ptr nonnull %_8.i)
  store ptr @_ZN1b4main17he022c5120c0a591eE, ptr %_8.i, align 8
; call std::rt::lang_start_internal
  %3 = call noundef i64 @_ZN3std2rt19lang_start_internal17h56699df87ce50e6fE(ptr noundef nonnull align 1 %_8.i, ptr noalias noundef nonnull readonly align 8 dereferenceable(24) @vtable.0, i64 noundef %2, ptr noundef %1, i8 noundef 0)
  call void @llvm.lifetime.end.p0(i64 8, ptr nonnull %_8.i)
  %4 = trunc i64 %3 to i32
  ret i32 %4
}

; Function Attrs: mustprogress nocallback nofree nosync nounwind willreturn memory(argmem: readwrite)
declare void @llvm.lifetime.start.p0(i64 immarg, ptr nocapture) #5

; Function Attrs: mustprogress nocallback nofree nosync nounwind willreturn memory(argmem: readwrite)
declare void @llvm.lifetime.end.p0(i64 immarg, ptr nocapture) #5

attributes #0 = { noinline nonlazybind uwtable "probe-stack"="inline-asm" "target-cpu"="x86-64" }
attributes #1 = { nonlazybind uwtable "probe-stack"="inline-asm" "target-cpu"="x86-64" }
attributes #2 = { inlinehint nonlazybind uwtable "probe-stack"="inline-asm" "target-cpu"="x86-64" }
attributes #3 = { inlinehint mustprogress nofree norecurse nosync nounwind nonlazybind willreturn memory(none) uwtable "probe-stack"="inline-asm" "target-cpu"="x86-64" }
attributes #4 = { nonlazybind "probe-stack"="inline-asm" "target-cpu"="x86-64" }
attributes #5 = { mustprogress nocallback nofree nosync nounwind willreturn memory(argmem: readwrite) }
attributes #6 = { nounwind }

!llvm.module.flags = !{!0, !1, !2}
!llvm.ident = !{!3}

!0 = !{i32 8, !"PIC Level", i32 2}
!1 = !{i32 7, !"PIE Level", i32 2}
!2 = !{i32 2, !"RtLibUseGOT", i32 1}
!3 = !{!"rustc version 1.74.0-nightly (58e967a9c 2023-09-03)"}
!4 = !{i32 1385354}
!5 = !{}
!6 = !{!7}
!7 = distinct !{!7, !8, !"_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h0ca664f915dfe515E: %_1"}
!8 = distinct !{!8, !"_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h0ca664f915dfe515E"}
