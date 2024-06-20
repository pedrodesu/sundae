; ModuleID = 'basic'
source_filename = "basic"

declare void @putd(i32)

define i32 @add(i32 %0, i32 %1) {
entry:
  %a = alloca i32, align 4
  store i32 %0, ptr %a, align 4
  %b = alloca i32, align 4
  store i32 %1, ptr %b, align 4
  %sum = add i32 %0, %1
  ret i32 %sum
}

define i32 @main() {
entry:
  %a = alloca i32, align 4
  store i32 40, ptr %a, align 4
  %b = alloca i32, align 4
  store i32 32, ptr %b, align 4
  %cast = load i32, ptr %a, align 4
  %cast1 = load i32, ptr %b, align 4
  %call = call i32 @add(i32 %cast, i32 %cast1)
  call void @putd(i32 %call)
  ret i32 0
}
