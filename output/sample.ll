; ModuleID = 'sample'
source_filename = "sample"

declare void @putd(i32)

define i32 @main() {
entry:
  %a = alloca i32, align 4
  store i32 10, ptr %a, align 4
  %b = alloca i32, align 4
  store i32 32, ptr %b, align 4
  ret i32 0
}
