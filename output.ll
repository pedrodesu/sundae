; ModuleID = 'sample'
source_filename = "sample"

define i32 @fib(i32 %0) {
entry:
  %sub = sub i32 %0, 1
  %call = call i32 @fib(i32 %sub)
  %sub1 = sub i32 %0, 2
  %call2 = call i32 @fib(i32 %sub1)
  %sum = add i32 %call, %call2
  ret i32 %sum
}

define void @main() {
entry:
  %call = call i32 @fib(i32 5)
}
