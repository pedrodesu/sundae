; ModuleID = 'sample'
source_filename = "sample"

declare void @putd(i32)

define void @swap(ptr %0, ptr %1) {
entry:
  %a = alloca ptr, align 8
  store ptr %0, ptr %a, align 8
  %b = alloca ptr, align 8
  store ptr %1, ptr %b, align 8
  %c = alloca i32, align 4
  %a1 = load ptr, ptr %a, align 8
  %deref = load i32, ptr %a1, align 4
  store i32 %deref, ptr %c, align 4
  ret void
}

define i32 @op(i32 %0, i32 %1) {
entry:
  %a = alloca i32, align 4
  store i32 %0, ptr %a, align 4
  %b = alloca i32, align 4
  store i32 %1, ptr %b, align 4
  %a1 = load i32, ptr %a, align 4
  %b2 = load i32, ptr %b, align 4
  %mul = mul i32 %a1, %b2
  ret i32 %mul
}

define void @main() {
entry:
  %a = alloca i32, align 4
  %call = call i32 @op(i32 10, i32 4)
  store i32 %call, ptr %a, align 4
  %b = alloca i32, align 4
  store i32 32, ptr %b, align 4
  %a1 = load i32, ptr %a, align 4
  %ref = alloca i32, align 4
  store i32 %a1, ptr %ref, align 4
  %b2 = load i32, ptr %b, align 4
  %ref3 = alloca i32, align 4
  store i32 %b2, ptr %ref3, align 4
  call void @swap(ptr %ref, ptr %ref3)
  %a4 = load i32, ptr %a, align 4
  call void @putd(i32 %a4)
  %b5 = load i32, ptr %b, align 4
  call void @putd(i32 %b5)
  ret void
}
