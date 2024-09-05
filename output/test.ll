; ModuleID = 'test'
source_filename = "test"

declare void @putd(i32)

define i32 @op(i32 %0, i32 %1) {
entry:
  %a = alloca i32, align 4
  store i32 %0, ptr %a, align 4
  %b = alloca i32, align 4
  store i32 %1, ptr %b, align 4
  %c = alloca i32, align 4
  store i32 7, ptr %c, align 4
  %d = alloca i32, align 4
  store i32 40, ptr %d, align 4
  %cast = load i32, ptr %c, align 4
  %cast1 = load i32, ptr %d, align 4
  %sum = add i32 %cast, %cast1
  %sum2 = add i32 %sum, %0
  %sum3 = add i32 %sum2, %1
  ret i32 %sum3
}

define i32 @main() {
entry:
  %call = call i32 @op(i32 2, i32 4)
  call void @putd(i32 %call)
  ret i32 0
}
