#![cfg(test)]

mod col_refs;
mod push;
mod slice;
mod splice;
mod string;
mod unshift;

use expect_test::expect;
use serde::de::Deserialize;
use std::collections::HashMap;

mod fixtures {
    use super::*;

    pub fn pk1() -> serde_json::Value {
        serde_json::json!({
            "kty": "EC",
            "crv": "secp256k1",
            "alg": "ES256K",
            "use": "sig",
            "x": "nnzHFO4bZ239bIuAo8t0wQwXH3fPwbKQnpWPzOptv0Q=",
            "y": "Z1-oY62A6q5kCRGfBuk6E3IrSUjPCK2F6_EwVhW22lY="
        })
    }

    pub fn pk1_key() -> abi::publickey::Key {
        abi::publickey::Key::deserialize(pk1()).unwrap()
    }

    pub fn pk2() -> serde_json::Value {
        serde_json::json!({
            "kty": "EC",
            "crv": "secp256k1",
            "alg": "ES256K",
            "use": "sig",
            "x": "nnzHFO4bZ239bIuAo8t0wQwXH3fPwbKQnpWPzOptv0Q=",
            // Z at the start was changed to Y
            "y": "Y1-oY62A6q5kCRGfBuk6E3IrSUjPCK2F6_EwVhW22lY="
        })
    }

    pub fn pk2_key() -> abi::publickey::Key {
        abi::publickey::Key::deserialize(pk2()).unwrap()
    }
}

macro_rules! consistency_checks {
    ($output:expr, $abi:expr, hashes: $hashes_expect:expr, dependencies: $dependencies_expect:expr) => {{
        let expected_hashes = $hashes_expect;
        expected_hashes.assert_debug_eq(&$output.hashes());

        let expected_dependencies = $dependencies_expect;
        expected_dependencies.assert_debug_eq(&$abi.dependent_fields);
    }};
}

fn run(
    polylang_code: &str,
    contract: &str,
    function: &str,
    this: serde_json::Value,
    args: Vec<serde_json::Value>,
    ctx_public_key: Option<abi::publickey::Key>,
    other_records: HashMap<String, Vec<serde_json::Value>>,
) -> Result<(abi::Abi, polylang_prover::RunOutput), error::Error> {
    let program = polylang::parse_program(polylang_code).unwrap();

    let (miden_code, abi) = polylang::compiler::compile(program, Some(contract), function)?;

    let program = polylang_prover::compile_program(&abi, &miden_code).unwrap();
    let inputs = polylang_prover::Inputs::new(
        abi.clone(),
        ctx_public_key,
        match &abi.this_type {
            Some(abi::Type::Struct(s)) => s.fields.iter().map(|_| 0).collect(),
            _ => unreachable!(),
        },
        this,
        args,
        {
            let mut hm = HashMap::new();
            for (contract, records) in other_records {
                let col = abi
                    .other_contract_types
                    .iter()
                    .find_map(|t| match t {
                        abi::Type::Struct(s) if s.name == contract => Some(s),
                        _ => None,
                    })
                    .unwrap();

                hm.insert(
                    contract,
                    records
                        .into_iter()
                        .map(|record| {
                            (
                                record.clone(),
                                col.fields.iter().map(|_| 0).collect::<Vec<_>>(),
                            )
                        })
                        .collect(),
                );
            }
            hm
        },
    )?;

    let (output, _) = polylang_prover::run(&program, &inputs)?;

    Ok((abi, output))
}

#[test]
fn call_public_collection() {
    let code = r#"
        collection Account {
            id: string;
            name: string;

            setName(name: string) {
                this.name = name;
            }
        }
    "#;

    let (abi, output) = run(
        code,
        "Account",
        "setName",
        serde_json::json!({
            "id": "",
            "name": "",
        }),
        vec![serde_json::json!("test")],
        None,
        HashMap::new(),
    )
    .unwrap();

    assert_eq!(
        output.this(&abi).unwrap(),
        abi::Value::StructValue(vec![
            ("id".to_owned(), abi::Value::String("".to_owned())),
            ("name".to_owned(), abi::Value::String("test".to_owned())),
        ])
    );

    consistency_checks!(
        output,
        abi,
        hashes:
            expect![[r#"
                []
            "#]],
        dependencies:
            expect![[r#"
                []
            "#]]
    );
}

#[test]
fn call_public_contract() {
    let code = r#"
        contract Account {
            id: string;
            name: string;

            setName(name: string) {
                this.name = name;
            }
        }
    "#;

    let (abi, output) = run(
        code,
        "Account",
        "setName",
        serde_json::json!({
            "id": "",
            "name": "",
        }),
        vec![serde_json::json!("test")],
        None,
        HashMap::new(),
    )
    .unwrap();

    assert_eq!(
        output.this(&abi).unwrap(),
        abi::Value::StructValue(vec![
            ("id".to_owned(), abi::Value::String("".to_owned())),
            ("name".to_owned(), abi::Value::String("test".to_owned())),
        ])
    );

    consistency_checks!(
        output,
        abi,
        hashes:
            expect![[r#"
                []
            "#]],
        dependencies:
            expect![[r#"
                []
            "#]]
    );
}

#[test]
fn call_any_call_collection() {
    let code = r#"
        @call
        collection Account {
            id: string;
            name: string;

            setName(name: string) {
                this.name = name;
            }
        }
    "#;

    let (abi, output) = run(
        code,
        "Account",
        "setName",
        serde_json::json!({
            "id": "",
            "name": "",
        }),
        vec![serde_json::json!("test")],
        None,
        HashMap::new(),
    )
    .unwrap();

    assert_eq!(
        output.this(&abi).unwrap(),
        abi::Value::StructValue(vec![
            ("id".to_owned(), abi::Value::String("".to_owned())),
            ("name".to_owned(), abi::Value::String("test".to_owned())),
        ])
    );

    consistency_checks!(
        output,
        abi,
        hashes:
            expect![[r#"
                []
            "#]],
        dependencies:
            expect![[r#"
                []
            "#]]
    );
}

#[test]
fn call_any_call_contract() {
    let code = r#"
        @call
        contract Account {
            id: string;
            name: string;

            setName(name: string) {
                this.name = name;
            }
        }
    "#;

    let (abi, output) = run(
        code,
        "Account",
        "setName",
        serde_json::json!({
            "id": "",
            "name": "",
        }),
        vec![serde_json::json!("test")],
        None,
        HashMap::new(),
    )
    .unwrap();

    assert_eq!(
        output.this(&abi).unwrap(),
        abi::Value::StructValue(vec![
            ("id".to_owned(), abi::Value::String("".to_owned())),
            ("name".to_owned(), abi::Value::String("test".to_owned())),
        ])
    );

    consistency_checks!(
        output,
        abi,
        hashes:
            expect![[r#"
                []
            "#]],
        dependencies:
            expect![[r#"
                []
            "#]]
    );
}

#[test]
fn call_constructor_no_auth() {
    let code = r#"
        contract Account {
            id: string;

            constructor (id: string) {
                this.id = id;
            }
        }
    "#;

    let (abi, output) = run(
        code,
        "Account",
        "constructor",
        serde_json::json!({
            "id": "",
        }),
        vec![serde_json::json!("id1")],
        None,
        HashMap::new(),
    )
    .unwrap();

    assert_eq!(
        output.this(&abi).unwrap(),
        abi::Value::StructValue(vec![(
            "id".to_owned(),
            abi::Value::String("id1".to_owned())
        )])
    );

    consistency_checks!(
        output,
        abi,
        hashes:
            expect![[r#"
                []
            "#]],
        dependencies:
            expect![[r#"
                []
            "#]]
    );
}

#[test]
fn call_constructor_with_auth() {
    let code = r#"
        contract Account {
            id: string;
            pk: PublicKey;

            constructor (id: string) {
                this.id = id;
                if (ctx.publicKey)
                    this.pk = ctx.publicKey;
                else error("missing public key");
            }
        }
    "#;

    let (abi, output) = run(
        code,
        "Account",
        "constructor",
        serde_json::json!({
            "id": "",
            "pk": fixtures::pk2(),
        }),
        vec![serde_json::json!("id1")],
        Some(fixtures::pk1_key()),
        HashMap::new(),
    )
    .unwrap();

    assert_eq!(
        output.this(&abi).unwrap(),
        abi::Value::StructValue(vec![
            ("id".to_owned(), abi::Value::String("id1".to_owned())),
            ("pk".to_owned(), abi::Value::PublicKey(fixtures::pk1_key())),
        ])
    );

    consistency_checks!(
        output,
        abi,
        hashes:
            expect![[r#"
                [
                    [
                        15190310144854117473,
                        13483436742372640428,
                        16238764937440726588,
                        9860411171209566744,
                    ],
                ]
            "#]],
        dependencies:
            expect![[r#"
                [
                    (
                        "pk",
                        PublicKey,
                    ),
                ]
            "#]]
    );
}

fn call_auth_public_key(use_correct_pk: bool) -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        contract Account {
            id: string;
            pk: PublicKey;

            constructor (id: string, pk: PublicKey) {
                this.id = id;
                this.pk = pk;
            }

            @call(pk)
            changePk(newPk: PublicKey) {
                this.pk = newPk;
            }
        }
    "#;

    let old_pk = fixtures::pk1();
    let old_pk_key = fixtures::pk1_key();
    let new_pk = fixtures::pk2();
    let new_pk_key = fixtures::pk2_key();

    let (abi, output) = run(
        code,
        "Account",
        "changePk",
        serde_json::json!({
            "id": "test",
            "pk": old_pk,
        }),
        vec![new_pk],
        Some(if use_correct_pk {
            old_pk_key
        } else {
            new_pk_key.clone()
        }),
        HashMap::new(),
    )?;

    assert_eq!(
        output.this(&abi).unwrap(),
        abi::Value::StructValue(vec![
            ("id".to_owned(), abi::Value::String("".to_owned())),
            ("pk".to_owned(), abi::Value::PublicKey(new_pk_key)),
        ]),
    );

    consistency_checks!(
        output,
        abi,
        hashes:
            expect![[r#"
                [
                    [
                        10007246358458628330,
                        1310941925803483469,
                        9098756844150300261,
                        7017043683864941931,
                    ],
                ]
            "#]],
        dependencies:
            expect![[r#"
                [
                    (
                        "pk",
                        PublicKey,
                    ),
                ]
            "#]]
    );

    Ok(())
}

#[test]
fn call_auth_public_key_correct_pk() {
    call_auth_public_key(true).unwrap();
}

#[test]
fn call_auth_public_key_wrong_pk() {
    let err = call_auth_public_key(false).unwrap_err();
    assert!(err
        .to_string()
        .contains("You are not authorized to call this function"));
}

#[test]
fn call_auth_public_key_no_pk() {
    let code = r#"
        contract Account {
            id: string;
            pk: PublicKey;

            constructor (id: string, pk: PublicKey) {
                this.id = id;
                this.pk = pk;
            }

            @call(pk)
            changePk(newPk: PublicKey) {
                this.pk = newPk;
            }
        }
    "#;

    let err = run(
        code,
        "Account",
        "changePk",
        serde_json::json!({
            "id": "test",
            "pk": fixtures::pk1(),
        }),
        vec![fixtures::pk2()],
        None,
        HashMap::new(),
    )
    .unwrap_err();

    assert!(err
        .to_string()
        .contains("You are not authorized to call this function"));
}

#[test]
fn call_auth_public_key_allow_all() {
    let code = r#"
        contract Account {
            id: string;
            pk: PublicKey;

            constructor (id: string, pk: PublicKey) {
                this.id = id;
                this.pk = pk;
            }

            @call
            changePk(newPk: PublicKey) {
                this.pk = newPk;
            }
        }
    "#;

    let (abi, output) = run(
        code,
        "Account",
        "changePk",
        serde_json::json!({
            "id": "test",
            "pk": fixtures::pk1(),
        }),
        vec![fixtures::pk2()],
        None,
        HashMap::new(),
    )
    .unwrap();

    assert_eq!(
        output.this(&abi).unwrap(),
        abi::Value::StructValue(vec![
            ("id".to_owned(), abi::Value::String("".to_owned())),
            ("pk".to_owned(), abi::Value::PublicKey(fixtures::pk2_key())),
        ]),
    );

    consistency_checks!(
        output,
        abi,
        hashes:
            expect![[r#"
                [
                    [
                        10007246358458628330,
                        1310941925803483469,
                        9098756844150300261,
                        7017043683864941931,
                    ],
                ]
            "#]],
        dependencies:
            expect![[r#"
                [
                    (
                        "pk",
                        PublicKey,
                    ),
                ]
            "#]]
    );
}

#[test]
fn call_auth_no_directive() {
    let code = r#"
        @private
        contract Account {
            id: string;
            pk: PublicKey;

            constructor (id: string, pk: PublicKey) {
                this.id = id;
                this.pk = pk;
            }

            changePk(newPk: PublicKey) {
                this.pk = newPk;
            }
        }
    "#;

    let err = run(
        code,
        "Account",
        "changePk",
        serde_json::json!({
            "id": "test",
            "pk": fixtures::pk1(),
        }),
        vec![fixtures::pk2()],
        None,
        HashMap::new(),
    )
    .unwrap_err();

    assert!(err
        .to_string()
        .contains("You are not authorized to call this function"));
}

#[test]
fn call_contract_auth_any() {
    let code = r#"
        @call
        contract Account {
            id: string;
            pk: PublicKey;

            changePk(newPk: PublicKey) {
                this.pk = newPk;
            }
        }
    "#;

    let (abi, output) = run(
        code,
        "Account",
        "changePk",
        serde_json::json!({
            "id": "test",
            "pk": fixtures::pk1(),
        }),
        vec![fixtures::pk2()],
        None,
        HashMap::new(),
    )
    .unwrap();

    assert_eq!(
        output.this(&abi).unwrap(),
        abi::Value::StructValue(vec![
            ("id".to_owned(), abi::Value::String("".to_owned())),
            ("pk".to_owned(), abi::Value::PublicKey(fixtures::pk2_key())),
        ]),
    );

    consistency_checks!(
        output,
        abi,
        hashes:
            expect![[r#"
                [
                    [
                        10007246358458628330,
                        1310941925803483469,
                        9098756844150300261,
                        7017043683864941931,
                    ],
                ]
            "#]],
        dependencies:
            expect![[r#"
                [
                    (
                        "pk",
                        PublicKey,
                    ),
                ]
            "#]]
    );
}

fn call_auth_delegate(use_correct_pk: bool) -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        contract User {
            id: string;
            @delegate
            pk: PublicKey;
        }

        contract Account {
            id: string;
            name: string;
            user: User;

            @call(user)
            changeName(name: string) {
                this.name = name;
            }
        }
    "#;

    let (abi, output) = run(
        code,
        "Account",
        "changeName",
        serde_json::json!({
            "id": "test",
            "name": "test",
            "user": {
                "id": "user1",
                "pk": fixtures::pk1(),
            },
        }),
        vec![serde_json::json!("test2")],
        Some(if use_correct_pk {
            fixtures::pk1_key()
        } else {
            fixtures::pk2_key()
        }),
        {
            let mut hm = HashMap::new();
            hm.insert(
                "User".to_owned(),
                vec![serde_json::json!({
                    "id": "user1",
                    "pk": fixtures::pk1(),
                })],
            );
            hm
        },
    )?;

    assert_eq!(
        output.this(&abi).unwrap(),
        abi::Value::StructValue(vec![
            ("id".to_owned(), abi::Value::String("".to_owned())),
            ("name".to_owned(), abi::Value::String("test2".to_owned())),
            (
                "user".to_owned(),
                abi::Value::ContractReference("user1".bytes().collect()),
            ),
        ]),
    );

    consistency_checks!(
        output,
        abi,
        hashes:
            expect![[r#"
                [
                    [
                        11000463426052588238,
                        6513620181524223329,
                        8307048643396104721,
                        12256912913701141453,
                    ],
                ]
            "#]],
        dependencies:
            expect![[r#"
                [
                    (
                        "user",
                        ContractReference {
                            contract: "User",
                        },
                    ),
                ]
            "#]]
    );

    Ok(())
}

#[test]
fn call_auth_delegate_correct_pk() {
    call_auth_delegate(true).unwrap();
}

#[test]
fn call_auth_delegate_wrong_pk() {
    let err = call_auth_delegate(false).unwrap_err();
    assert!(err
        .to_string()
        .contains("You are not authorized to call this function"));
}

fn call_auth_literal_pk(use_correct_pk: bool) -> Result<(), Box<dyn std::error::Error>> {
    let key = fixtures::pk1_key().to_64_byte_hex();
    let code = format!(
        r#"
        contract Account {{
            id: string;
            name: string;

            @call(eth#{key})
            changeName(name: string) {{
                this.name = name;
            }}
        }}
    "#
    );

    let (abi, output) = run(
        &code,
        "Account",
        "changeName",
        serde_json::json!({
            "id": "test",
            "name": "test",
        }),
        vec![serde_json::json!("test2")],
        Some(if use_correct_pk {
            fixtures::pk1_key()
        } else {
            fixtures::pk2_key()
        }),
        HashMap::new(),
    )?;

    assert_eq!(
        output.this(&abi).unwrap(),
        abi::Value::StructValue(vec![
            ("id".to_owned(), abi::Value::String("".to_owned())),
            ("name".to_owned(), abi::Value::String("test2".to_owned())),
        ]),
    );

    consistency_checks!(
        output,
        abi,
        hashes:
            expect![[r#"
                []
            "#]],
        dependencies:
            expect![[r#"
                []
            "#]]
    );

    Ok(())
}

#[test]
fn call_auth_literal_pk_correct_pk() {
    call_auth_literal_pk(true).unwrap();
}

#[test]
fn call_auth_literal_pk_wrong_pk() {
    let err = call_auth_literal_pk(false).unwrap_err();
    assert!(err
        .to_string()
        .contains("You are not authorized to call this function"));
}

#[test]
fn call_auth_literal_compressed() {
    let key = fixtures::pk1_key().to_compressed_33_byte_hex();
    let code = format!(
        r#"
        contract Account {{
            id: string;
            name: string;

            @call(eth#{key})
            changeName(name: string) {{
                this.name = name;
            }}
        }}
    "#
    );

    let (abi, output) = run(
        &code,
        "Account",
        "changeName",
        serde_json::json!({
            "id": "test",
            "name": "test",
        }),
        vec![serde_json::json!("test2")],
        Some(fixtures::pk1_key()),
        HashMap::new(),
    )
    .unwrap();

    assert_eq!(
        output.this(&abi).unwrap(),
        abi::Value::StructValue(vec![
            ("id".to_owned(), abi::Value::String("".to_owned())),
            ("name".to_owned(), abi::Value::String("test2".to_owned())),
        ]),
    );

    consistency_checks!(
        output,
        abi,
        hashes:
            expect![[r#"
                []
            "#]],
        dependencies:
            expect![[r#"
                []
            "#]]
    );
}

#[test]
fn read_auth_field_correct_ctx() {
    let code = r#"
        @private
        contract Account {
            id: string;
            @read
            pk: PublicKey;
        }
    "#;

    let (abi, output) = run(
        code,
        "Account",
        ".readAuth",
        serde_json::json!({
            "id": "",
            "pk": fixtures::pk1(),
        }),
        vec![],
        Some(fixtures::pk1_key()),
        HashMap::new(),
    )
    .unwrap();

    assert!(output.read_auth());

    consistency_checks!(
        output,
        abi,
        hashes:
            expect![[r#"
                [
                    [
                        15190310144854117473,
                        13483436742372640428,
                        16238764937440726588,
                        9860411171209566744,
                    ],
                ]
            "#]],
        dependencies:
            expect![[r#"
                [
                    (
                        "pk",
                        PublicKey,
                    ),
                ]
            "#]]
    );
}

#[test]
fn read_auth_field_wrong_ctx() {
    let code = r#"
        @private
        contract Account {
            id: string;
            @read
            pk: PublicKey;
        }
    "#;

    let (_, output) = run(
        code,
        "Account",
        ".readAuth",
        serde_json::json!({
            "id": "",
            "pk": fixtures::pk1(),
        }),
        vec![],
        Some(fixtures::pk2_key()),
        HashMap::new(),
    )
    .unwrap();

    assert!(!output.read_auth());
}

#[test]
fn read_auth_field_no_ctx() {
    let code = r#"
        @private
        contract Account {
            id: string;
            @read
            pk: PublicKey;
        }
    "#;

    let (abi, output) = run(
        code,
        "Account",
        ".readAuth",
        serde_json::json!({
            "id": "",
            "pk": fixtures::pk1(),
        }),
        vec![],
        None,
        HashMap::new(),
    )
    .unwrap();

    assert!(!output.read_auth());

    consistency_checks!(
        output,
        abi,
        hashes:
            expect![[r#"
                [
                    [
                        15190310144854117473,
                        13483436742372640428,
                        16238764937440726588,
                        9860411171209566744,
                    ],
                ]
            "#]],
        dependencies:
            expect![[r#"
                [
                    (
                        "pk",
                        PublicKey,
                    ),
                ]
            "#]]
    );
}

#[test]
fn read_auth_contract_with_pk() {
    let code = r#"
        @read
        contract Account {
            id: string;
            pk: PublicKey;
        }
    "#;

    let (abi, output) = run(
        code,
        "Account",
        ".readAuth",
        serde_json::json!({
            "id": "",
            "pk": fixtures::pk1(),
        }),
        vec![],
        Some(fixtures::pk1_key()),
        HashMap::new(),
    )
    .unwrap();

    assert!(output.read_auth());

    consistency_checks!(
        output,
        abi,
        hashes:
            expect![[r#"
                [
                    [
                        15190310144854117473,
                        13483436742372640428,
                        16238764937440726588,
                        9860411171209566744,
                    ],
                ]
            "#]],
        dependencies:
            expect![[r#"
                [
                    (
                        "pk",
                        PublicKey,
                    ),
                ]
            "#]]
    );
}

#[test]
fn read_auth_contract_without_pk() {
    let code = r#"
        @read
        contract Account {
            id: string;
            pk: PublicKey;
        }
    "#;

    let (abi, output) = run(
        code,
        "Account",
        ".readAuth",
        serde_json::json!({
            "id": "",
            "pk": fixtures::pk1(),
        }),
        vec![],
        None,
        HashMap::new(),
    )
    .unwrap();

    assert!(output.read_auth());

    consistency_checks!(
        output,
        abi,
        hashes:
            expect![[r#"
                [
                    [
                        15190310144854117473,
                        13483436742372640428,
                        16238764937440726588,
                        9860411171209566744,
                    ],
                ]
            "#]],
        dependencies:
            expect![[r#"
                [
                    (
                        "pk",
                        PublicKey,
                    ),
                ]
            "#]]
    );
}

#[test]
fn field_hashes() {
    let code = r#"
        contract Account {
            id: string;
            balance: u32;

            addBalance(amount: u32) {
                this.balance = this.balance + amount;
            }
        }
    "#;

    let (abi, output) = run(
        code,
        "Account",
        "addBalance",
        serde_json::json!({
            "id": "john",
            "balance": 0,
        }),
        vec![serde_json::json!(10)],
        None,
        HashMap::new(),
    )
    .unwrap();

    assert_eq!(
        output.this(&abi).unwrap(),
        abi::Value::StructValue(vec![
            ("id".to_owned(), abi::Value::String("".to_owned())),
            ("balance".to_owned(), abi::Value::UInt32(10)),
        ])
    );

    consistency_checks!(
        output,
        abi,
        hashes:
            expect![[r#"
                [
                    [
                        10272219061387384304,
                        13401779264242975131,
                        10013658661959349609,
                        9575923678792186484,
                    ],
                ]
            "#]],
        dependencies:
            expect![[r#"
            [
                (
                    "balance",
                    PrimitiveType(
                        UInt32,
                    ),
                ),
            ]
        "#]]
    );
}

#[test]
fn field_dependencies() {
    let code = r#"
        contract Account {
            id: string;
            name: string;
            balance: u32;

            addBalance(amount: u32) {
                this.balance = this.balance + amount;
            }
        }
    "#;

    let (abi, output) = run(
        code,
        "Account",
        "addBalance",
        serde_json::json!({
            "id": "john",
            "name": "John Doe",
            "balance": 0,
        }),
        vec![serde_json::json!(10)],
        None,
        HashMap::new(),
    )
    .unwrap();

    assert_eq!(
        output.this(&abi).unwrap(),
        abi::Value::StructValue(vec![
            ("id".to_owned(), abi::Value::String("".to_owned())), // id was not passed to the VM
            ("name".to_owned(), abi::Value::String("".to_owned())), // name was not passed to the VM
            ("balance".to_owned(), abi::Value::UInt32(10)),
        ])
    );

    consistency_checks!(
        output,
        abi,
        hashes:
            expect![[r#"
                [
                    [
                        10272219061387384304,
                        13401779264242975131,
                        10013658661959349609,
                        9575923678792186484,
                    ],
                ]
            "#]],
        dependencies:
            expect![[r#"
            [
                (
                    "balance",
                    PrimitiveType(
                        UInt32,
                    ),
                ),
            ]
        "#]]
    );
}

#[test]
fn index_of() {
    fn run_index_of(
        element_type: &str,
        arr: Vec<serde_json::Value>,
        item: serde_json::Value,
    ) -> Result<abi::Value, error::Error> {
        let code = r#"
                contract Account {
                id: string;
                result: i32;

                indexOf(arr: $ELEMENT_TYPE[], item: $ELEMENT_TYPE) {
                    this.result = arr.indexOf(item);
                }
            }
        "#
        .replace("$ELEMENT_TYPE", element_type);

        let (abi, output) = run(
            &code,
            "Account",
            "indexOf",
            serde_json::json!({
                "id": "test",
                "result": 123456,
            }),
            vec![serde_json::json!(arr), serde_json::json!(item)],
            None,
            HashMap::new(),
        )?;

        let this = output.this(&abi).unwrap();
        Ok(match this {
            abi::Value::StructValue(fields) => fields
                .into_iter()
                .find_map(|(k, v)| if k == "result" { Some(v) } else { None })
                .unwrap(),
            _ => unreachable!(),
        })
    }

    assert_eq!(
        run_index_of(
            "string",
            vec![serde_json::json!("a"), serde_json::json!("b")],
            serde_json::json!("a")
        )
        .unwrap(),
        abi::Value::Int32(0),
    );

    assert_eq!(
        run_index_of(
            "string",
            vec![serde_json::json!("a"), serde_json::json!("b")],
            serde_json::json!("b")
        )
        .unwrap(),
        abi::Value::Int32(1),
    );

    assert_eq!(
        run_index_of(
            "string",
            vec![serde_json::json!("a"), serde_json::json!("b")],
            serde_json::json!("c")
        )
        .unwrap(),
        abi::Value::Int32(-1),
    );

    assert_eq!(
        run_index_of(
            "i32",
            vec![serde_json::json!(1), serde_json::json!(2)],
            serde_json::json!(2)
        )
        .unwrap(),
        abi::Value::Int32(1),
    );
}

#[test]
fn returning() {
    let code = r#"
        @public
        contract Account {
            id: string;
            name: string;


            @call
            getName(): string {
                return this.name;
            }
        }
    "#;

    let (abi, output) = run(
        code,
        "Account",
        "getName",
        serde_json::json!({
            "id": "",
            "name": "John",
        }),
        vec![],
        None,
        HashMap::new(),
    )
    .unwrap();

    assert_eq!(
        output.result(&abi).unwrap(),
        abi::Value::String("John".to_owned())
    );

    consistency_checks!(
        output,
        abi,
        hashes:
            expect![[r#"
                [
                    [
                        13624894021325080686,
                        17508316994659426056,
                        2620902411312551361,
                        8347809021004383550,
                    ],
                ]
            "#]],
        dependencies:
            expect![[r#"
                [
                    (
                        "name",
                        String,
                    ),
                ]
            "#]]
    );

    let expected_return_hash = expect![[r#"
        [
            3850899504691128854,
            5609950875147406075,
            16200726738043745980,
            18156263845020900466,
        ]
    "#]];
    expected_return_hash.assert_debug_eq(&output.result_hash(&abi).unwrap());
}
