; ModuleID = 'sample'
source_filename = "sample"

define i32 @fib(i32 %0) {
entry:
}

define void @main() {
entry:
  %call = call i32 @fib(i32 5)
}
