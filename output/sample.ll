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
  store ptr %0, ptr %c, align 8
  store ptr %1, ptr %0, align 8
  store ptr %c, ptr %1, align 8
  ret void
}

define i32 @op(i32 %0, i32 %1) {
entry:
  %a = alloca i32, align 4
  store i32 %0, ptr %a, align 4
  %b = alloca i32, align 4
  store i32 %1, ptr %b, align 4
  %mul = mul i32 %0, %1
  ret i32 %mul
}

define i32 @main() {
entry:
  %a = alloca i32, align 4
  %call = call i32 @op(i32 10, i32 4)
  store i32 %call, ptr %a, align 4
  %cast = alloca i32, align 4
  store i32 32, ptr %cast, align 4
  call void @swap(ptr %a, ptr %cast)
  %cast1 = load i32, ptr %a, align 4
  call void @putd(i32 %cast1)
  ret i32 0
}
