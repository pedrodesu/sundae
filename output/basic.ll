; ModuleID = 'basic'
source_filename = "basic"

declare void @putd(i32)

define i32 @op() {
entry:
  ret i32 42
}

define i32 @main() {
entry:
  %a = alloca i32, align 4
  store i32 40, ptr %a, align 4
  %b = alloca i32, align 4
  store i32 32, ptr %b, align 4
  %call = call i32 @op()
  %c = alloca i32, align 4
  store ptr %a, ptr %c, align 8
  ret i32 0
}
