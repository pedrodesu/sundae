; ModuleID = 'sample'
source_filename = "sample"

@MAGIC_NUMBER = constant i32 42

declare void @putd(i32)

define void @swap(ptr %0, ptr %1) {
entry:
  %a = alloca ptr, align 8
  store ptr %0, ptr %a, align 8
  %b = alloca ptr, align 8
  store ptr %1, ptr %b, align 8
  %c = alloca i32, align 4
  %cast = load i32, ptr %0, align 4
  store i32 %cast, ptr %c, align 4
  %load = load i32, ptr %1, align 4
  store i32 %load, ptr %0, align 4
  %load1 = load i32, ptr %c, align 4
  store i32 %load1, ptr %1, align 4
  ret void
}

define i32 @op(i32 %0, i32 %1) {
entry:
  %a = alloca i32, align 4
  store i32 %0, ptr %a, align 4
  %b = alloca i32, align 4
  store i32 %1, ptr %b, align 4
  %gt = icmp sgt i32 %0, %1
  br i1 %gt, label %then, label %else

then:                                             ; preds = %entry
  %mul = mul i32 %0, %1
  ret i32 %mul

else:                                             ; preds = %entry
  %sum = add i32 %0, %1
  ret i32 %sum
}

define i32 @main() {
entry:
  %a = alloca i32, align 4
  %call = call i32 @op(i32 4, i32 10)
  store i32 %call, ptr %a, align 4
  %b = alloca i32, align 4
  store i32 42, ptr %b, align 4
  call void @swap(ptr %a, ptr %b)
  %cast = load i32, ptr %a, align 4
  call void @putd(i32 %cast)
  %cast1 = load i32, ptr %b, align 4
  call void @putd(i32 %cast1)
  ret i32 0
}
