(module
  (type (;0;) (func))
  (type (;1;) (func (param i32) (result i32)))
  (func $__wasm_call_ctors (type 0))
  (func $bool_to_str (type 1) (param i32) (result i32)
    i32.const 1024
    i32.const 1030
    local.get 0
    i32.const 1
    i32.and
    select)
  (func $fib (type 1) (param i32) (result i32)
    (local i32 i32 i32)
    i32.const 0
    local.set 1
    block  ;; label = @1
      block  ;; label = @2
        local.get 0
        i32.const 2
        i32.ge_s
        br_if 0 (;@2;)
        local.get 0
        local.set 2
        br 1 (;@1;)
      end
      i32.const 0
      local.set 1
      loop  ;; label = @2
        local.get 0
        i32.const -1
        i32.add
        call $fib
        local.get 1
        i32.add
        local.set 1
        local.get 0
        i32.const 4
        i32.lt_u
        local.set 3
        local.get 0
        i32.const -2
        i32.add
        local.tee 2
        local.set 0
        local.get 3
        i32.eqz
        br_if 0 (;@2;)
      end
    end
    local.get 2
    local.get 1
    i32.add)
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
  (export "bool_to_str" (func $bool_to_str))
  (export "fib" (func $fib))
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
