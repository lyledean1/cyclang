(module
  (type (;0;) (func))
  (type (;1;) (func (param i32) (result i32)))
  (func $__wasm_call_ctors (type 0))
  (func $main (type 0)
    (local i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 0
    global.set $__stack_pointer
    local.get 0
    i32.const 20
    i32.store offset=8
    local.get 0
    i32.const 20
    call $fib
    i32.store
    local.get 0
    i32.const 16
    i32.add
    global.set $__stack_pointer)
  (func $fib (type 1) (param i32) (result i32)
    (local i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 1
    global.set $__stack_pointer
    local.get 1
    local.tee 2
    i32.const 2
    i32.store offset=8
    local.get 2
    local.get 0
    i32.const 2
    i32.lt_s
    i32.store8 offset=7
    block  ;; label = @1
      local.get 0
      i32.const 1
      i32.gt_s
      br_if 0 (;@1;)
      local.get 2
      i32.const 16
      i32.add
      global.set $__stack_pointer
      local.get 0
      return
    end
    local.get 1
    i32.const -16
    i32.add
    local.tee 1
    local.tee 3
    global.set $__stack_pointer
    local.get 1
    i32.const 1
    i32.store
    local.get 3
    i32.const -16
    i32.add
    local.tee 1
    local.tee 3
    global.set $__stack_pointer
    local.get 1
    local.get 0
    i32.const -1
    i32.add
    local.tee 4
    i32.store
    local.get 4
    call $fib
    local.set 1
    local.get 3
    i32.const -16
    i32.add
    local.tee 3
    local.tee 4
    global.set $__stack_pointer
    local.get 3
    local.get 1
    i32.store
    local.get 4
    i32.const -16
    i32.add
    local.tee 3
    local.tee 4
    global.set $__stack_pointer
    local.get 3
    i32.const 2
    i32.store
    local.get 4
    i32.const -16
    i32.add
    local.tee 3
    local.tee 4
    global.set $__stack_pointer
    local.get 3
    local.get 0
    i32.const -2
    i32.add
    local.tee 0
    i32.store
    local.get 0
    call $fib
    local.set 0
    local.get 4
    i32.const -16
    i32.add
    local.tee 3
    local.tee 4
    global.set $__stack_pointer
    local.get 3
    local.get 0
    i32.store
    local.get 4
    i32.const -16
    i32.add
    local.tee 3
    global.set $__stack_pointer
    local.get 3
    local.get 1
    local.get 0
    i32.add
    local.tee 0
    i32.store
    local.get 2
    i32.const 16
    i32.add
    global.set $__stack_pointer
    local.get 0)
  (func $bool_to_str (type 1) (param i32) (result i32)
    block  ;; label = @1
      local.get 0
      i32.const 1
      i32.and
      i32.eqz
      br_if 0 (;@1;)
      i32.const 1024
      return
    end
    i32.const 1030)
  (memory (;0;) 2)
  (global $__stack_pointer (mut i32) (i32.const 66576))
  (global (;1;) i32 (i32.const 1024))
  (global (;2;) i32 (i32.const 1037))
  (global (;3;) i32 (i32.const 1040))
  (global (;4;) i32 (i32.const 66576))
  (global (;5;) i32 (i32.const 1024))
  (global (;6;) i32 (i32.const 66576))
  (global (;7;) i32 (i32.const 131072))
  (global (;8;) i32 (i32.const 0))
  (global (;9;) i32 (i32.const 1))
  (export "memory" (memory 0))
  (export "__wasm_call_ctors" (func $__wasm_call_ctors))
  (export "main" (func $main))
  (export "fib" (func $fib))
  (export "bool_to_str" (func $bool_to_str))
  (export "__dso_handle" (global 1))
  (export "__data_end" (global 2))
  (export "__stack_low" (global 3))
  (export "__stack_high" (global 4))
  (export "__global_base" (global 5))
  (export "__heap_base" (global 6))
  (export "__heap_end" (global 7))
  (export "__memory_base" (global 8))
  (export "__table_base" (global 9))
  (data $.rodata (i32.const 1024) "true\0a\00false\0a\00"))
