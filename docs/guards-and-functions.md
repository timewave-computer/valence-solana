# Guards and Functions in the Valence Microkernel

## Overview

The Valence microkernel implements a clear separation between authorization and execution through two fundamental primitives: guards and functions. This design ensures that security policies remain distinct from business logic while maintaining composability and verifiability throughout the system.

## Guards: The Authorization Layer

Guards serve as the authorization primitive in valence-kernel, acting as gatekeepers that determine whether an operation is allowed to execute. At their core, guards are pure predicates that evaluate to either true or false without producing any side effects. This purity is essential to the security model, as it ensures that the act of checking authorization cannot itself modify system state.

The guard system supports both built-in and external implementations. Built-in guards execute within a restricted virtual machine that provides common authorization patterns like ownership checks, usage limits, and time windows. These can be composed using boolean logic operators (And, Or, Not) to create sophisticated authorization policies from simple building blocks.

```rust
// Example of guard composition
let guard = Guard::And(
    Box::new(Guard::OwnerOnly),
    Box::new(Guard::TimeWindow { 
        start: now, 
        end: now + 3600 
    })
);
```

External guards enable custom authorization logic by delegating to user-deployed programs. However, these programs are constrained to read-only access, ensuring they cannot circumvent the purity requirement. The kernel enforces this through careful account permission management during cross-program invocations.

## Functions: The Execution Layer

While guards control access, functions in the valence-functions program define the actual computations and transformations. Functions are designed as deterministic transformations that map inputs to outputs without directly accessing or modifying blockchain state. This separation ensures that business logic remains testable, composable, and verifiable.

The function trait enforces purity through its type signature:

```rust
pub trait PureFunction {
    type Input: BorshDeserialize;
    type Output: BorshSerialize;
    type Error: Into<ProgramError>;
    
    fn execute(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
}
```

By working only with serialized input and output types rather than raw accounts, functions cannot directly read or write blockchain state. All state transitions must flow through the kernel, which maintains complete control over when and how changes occur. This design makes functions highly composable - they can be chained together in pipelines where the output of one becomes the input to the next.

## Purity Enforcement

The microkernel enforces purity through multiple mechanisms working in concert. For guards, the virtual machine provides a sandboxed execution environment with a limited instruction set that excludes any operations capable of modifying state. External guards receive only read permissions on accounts, enforced at the Solana runtime level through the account metadata passed during cross-program invocations.

Function purity emerges from architectural constraints rather than runtime enforcement. Since functions operate on deserialized data rather than raw accounts, they simply lack the capability to perform direct state modifications. Any attempt to include state-changing operations would fail at compile time due to type mismatches. This approach provides strong guarantees without runtime overhead.

The kernel itself acts as the sole arbiter of state transitions. When executing an operation batch, it first evaluates all relevant guards. Only if authorization succeeds does it proceed to invoke functions and apply their outputs to the blockchain state. This sequential evaluation ensures that no computation occurs without proper authorization.

## Architectural Relationships

The relationship between guards, functions, and the kernel follows a clear hierarchical pattern. Sessions in the kernel maintain references to compiled guard data and track authorization state. When an operation is requested, the kernel first evaluates the session's guards. If authorization succeeds, the kernel may invoke external functions through cross-program calls, passing serialized inputs and receiving outputs.

This architecture creates a unidirectional flow of control. The kernel orchestrates everything, guards make authorization decisions, and functions perform computations. No component can bypass this flow - functions cannot execute without guard approval, and guards cannot modify state regardless of their decision.

The separation extends to how these components are developed and deployed. Guards that implement common patterns ship with the valence-functions program as a library of reusable predicates. Custom business logic lives in separate function implementations that can be composed and registered independently. The kernel remains minimal, containing only the essential mechanisms for session management and operation execution.

## Design Philosophy

This architecture embodies the microkernel philosophy of providing mechanisms, not policies. The kernel offers the fundamental capability of guarded execution but makes no assumptions about what guards should check or what functions should compute. This neutrality enables the system to support diverse use cases without modification to the core protocol.

The strict separation between authorization and execution provides several benefits beyond security. It enables independent evolution of guards and functions, allows fine-grained composition of both authorization policies and business logic, and creates clear audit trails that distinguish between "what was checked" and "what was done."

By enforcing purity at the architectural level rather than through runtime checks, the system achieves both strong security guarantees and efficient execution. The constraints that ensure purity also promote good software engineering practices: guards remain focused on authorization logic, functions contain only business rules, and the kernel provides a minimal but complete orchestration layer.

## Practical Implications

For developers building on Valence, this architecture means thinking in terms of two distinct phases for every operation. First, define the authorization requirements as guards - who can perform this operation, under what conditions, with what limits? Second, implement the business logic as pure functions - what transformations should occur, how should data be validated and processed?

This separation often leads to more reusable code. A single guard policy might protect multiple different functions, and a single function might be used in contexts with different authorization requirements. The kernel's session mechanism provides the glue that binds specific guards to specific operations, enabling flexible composition without sacrificing security.

The purity constraints, while initially seeming restrictive, actually simplify development and testing. Pure functions can be unit tested with simple input/output assertions. Guard logic can be verified independently of the operations it protects. The kernel's orchestration logic remains minimal and auditable. This modularity reduces the surface area for bugs and makes the entire system more maintainable.

## Conclusion

The guard/function duality in the Valence microkernel represents a fundamental architectural decision that prioritizes security, composability, and verifiability. By strictly separating authorization from execution and enforcing purity through architectural constraints, the system provides a robust foundation for building complex protocols while maintaining strong security guarantees. This design enables developers to focus on their specific domain logic while relying on the kernel to ensure that all operations occur within properly authorized contexts.