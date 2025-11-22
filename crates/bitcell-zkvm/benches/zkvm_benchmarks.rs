use criterion::{black_box, criterion_group, criterion_main, Criterion};
use bitcell_zkvm::{Instruction, Interpreter, OpCode};

fn interpreter_arithmetic_benchmark(c: &mut Criterion) {
    c.bench_function("zkvm_arithmetic_100_ops", |b| {
        let mut program = Vec::new();
        for i in 0..100 {
            program.push(Instruction::new(OpCode::Add, (i % 32) as u8, 0, 1));
        }
        program.push(Instruction::new(OpCode::Halt, 0, 0, 0));
        
        b.iter(|| {
            let mut interp = Interpreter::new(10000);
            black_box(interp.execute(&program).unwrap())
        });
    });
}

fn interpreter_memory_benchmark(c: &mut Criterion) {
    c.bench_function("zkvm_memory_ops", |b| {
        let mut program = Vec::new();
        for i in 0..50 {
            program.push(Instruction::new(OpCode::Store, i as u8, 0, i * 10));
            program.push(Instruction::new(OpCode::Load, i as u8, 0, i * 10));
        }
        program.push(Instruction::new(OpCode::Halt, 0, 0, 0));
        
        b.iter(|| {
            let mut interp = Interpreter::new(10000);
            black_box(interp.execute(&program).unwrap())
        });
    });
}

fn interpreter_control_flow_benchmark(c: &mut Criterion) {
    c.bench_function("zkvm_control_flow", |b| {
        // Loop program: counter from 0 to 100
        let program = vec![
            Instruction::new(OpCode::Add, 0, 0, 1),    // r0++
            Instruction::new(OpCode::Lt, 1, 0, 100),   // r1 = r0 < 100
            Instruction::new(OpCode::Jz, 1, 0, 0),     // if r1 == 0, jump to 0
            Instruction::new(OpCode::Halt, 0, 0, 0),
        ];
        
        b.iter(|| {
            let mut interp = Interpreter::new(100000);
            black_box(interp.execute(&program).unwrap())
        });
    });
}

criterion_group!(
    benches,
    interpreter_arithmetic_benchmark,
    interpreter_memory_benchmark,
    interpreter_control_flow_benchmark
);
criterion_main!(benches);
