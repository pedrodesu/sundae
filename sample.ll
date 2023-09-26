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
  store ptr %a1, ptr %c, align 8
  %b2 = load ptr, ptr %b, align 8
  store ptr %b2, ptr %a, align 8
  %c3 = load i32, ptr %c, align 4
  store i32 %c3, ptr %b, align 4
  ret void
}

define void @main() {
entry:
  %a = alloca i32, align 4
  store i32 10, ptr %a, align 4
  %b = alloca i32, align 4
  store i32 32, ptr %b, align 4
  call void @swap(ptr %a, ptr %b)
  %a1 = load i32, ptr %a, align 4
  call void @putd(i32 %a1)
  %b2 = load i32, ptr %b, align 4
  call void @putd(i32 %b2)
  ret void
}
