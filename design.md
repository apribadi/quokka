# TYPES

## value types

- a value is an instance of a value type

- no identity

- heterogeneous representation
    - nothing
    - collection of scalars
    - tagged union??

- in Rust, would be "Copy"

- compiler generated value types have functions that serve as introduction and
  elimination forms

## object types

- an object is an instance of an object type

- identity

- in the abstract machine, lives in a fixed place in memory

- allocated on stack or heap

- a reference (pointer) to an object is a value

- garbage collected

- an object has fields, which may be
    - immutable value field
    - mutable value field
    - inlined object

- flexible array member

- some expressions evaluate to an object initializer, which is distinct from an
  object. desugars to taking an argument pointer to the uninitialized object

## function types

- multiple argument values

- multiple return values

- multiple return continuations

- can evaluate to an object initializer

## expression types

- correspond with the effects of evaluating a function, i.e. possible multiple
  values and multiple continuations

- can be an object initializer

## polymorphism

- prenex

## type based disambiguation

- operators

- field projection

- etc??

## type checking

- type checking an implementation file ONLY depends on other signature files

- implicit generalization of top level items

- ??

# IR

- SSA form

- implicit naming of program points, instructions, and values; i.e. uses
  integer indexes

- fixed-sized instructions to be in an array

- basic block declares number of input values and number of output values.
  instructions pop/put inputs/outputs

- a function call is a block terminator, takes continuation labels

# desugaring 

## evaluation order

- evaluate arguments left-to-right, then the head LAST

```
(#4)(#1, #2, #3)
```

## assignment to a place

```
x.foo = y

=>

__operator_set_foo(y, x)
```

```
x[y] = z

=>

__operator_set_index(z, y, x) # left-to-right evaluation order
```

## for loops

```
for x in e {
    ...
}

=>

case e(lambda (x) { ... }) {
    _ => ...
    `break => goto ...
    `continue => goto ...
    `return => goto ...
}

```
