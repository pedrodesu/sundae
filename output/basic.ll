; ModuleID = 'basic'
source_filename = "basic"

declare void @putd(i32)

define i32 @op(i32 %0, i32 %1) {
entry:
  %a = alloca i32, align 4
  store i32 %0, ptr %a, align 4
  %b = alloca i32, align 4
  store i32 %1, ptr %b, align 4
  %mul = mul ptr %a, %b
  ret ptr %mul
}

define i32 @main() {
entry:
  %b = alloca i32, align 4
  store i32 32, ptr %b, align 4
  %c = alloca i32, align 4
  store ptr %b, ptr %c, align 8
  ret i32 0
}
