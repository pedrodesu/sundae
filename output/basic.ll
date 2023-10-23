; ModuleID = 'basic'
source_filename = "basic"

declare void @putd(i32)

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
  %call2 = call void @putd(i32 %a1)
  %b3 = load i32, ptr %b, align 4
  %call4 = call void @putd(i32 %b3)
  ret void
}
