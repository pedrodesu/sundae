; ModuleID = 'sample'
source_filename = "sample"

define i32 @fib(i32 %0) {
entry:
  %le = icmp sle i32 %0, 1
  br i1 %le, label %then, label %else

then:                                             ; preds = %entry
  ret i32 %0
  br label %continue

else:                                             ; preds = %entry
  br label %continue

continue:                                         ; preds = %else, %then
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
