;; Simple SentientOS WebAssembly Application
(module
  ;; Import the function for printing to console
  (import "env" "print" (func $print (param i32 i32)))
  
  ;; Memory section
  (memory 1)
  
  ;; Data section - strings we'll print
  (data (i32.const 0) "Hello from inside SentientOS!\n")
  (data (i32.const 50) "Running in the burn environment\n")
  (data (i32.const 100) "Calculation result: ")
  
  ;; Function to run calculation
  (func $calculate (result i32)
    (i32.add
      (i32.mul
        (i32.const 42)
        (i32.const 10)
      )
      (i32.const 7)
    )
  )
  
  ;; Main function
  (func $main
    ;; Print first message
    (call $print
      (i32.const 0)
      (i32.const 30)
    )
    
    ;; Print second message
    (call $print
      (i32.const 50)
      (i32.const 31)
    )
    
    ;; Print calculation message
    (call $print
      (i32.const 100)
      (i32.const 19)
    )
    
    ;; Do calculation and print result
    (call $calculate)
    drop
  )
  
  ;; Export the main function
  (export "main" (func $main))
)
