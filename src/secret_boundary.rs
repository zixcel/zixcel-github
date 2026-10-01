use crate::ConnectorError;

pub(crate) fn reject_secrets(source: &str) -> Result<(), ConnectorError> {
    if source.len() > 1_048_576 {
        return Err(ConnectorError::new("config", "configuration exceeds 1 MiB"));
    }
    let value: toml::Value = toml::from_str(source)
        .map_err(|_| ConnectorError::new("config", "configuration is not valid TOML"))?;
    reject_value(&value, 0)
}

fn reject_value(value: &toml::Value, depth: usize) -> Result<(), ConnectorError> {
    if depth > 32 {
        return Err(ConnectorError::new(
            "config",
            "configuration nesting exceeds 32 levels",
        ));
    }
    match value {
        toml::Value::Table(table) => reject_table(table, depth),
        toml::Value::Array(items) => {
            for item in items {
                reject_value(item, depth + 1)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn reject_table(
    table: &toml::map::Map<String, toml::Value>,
    depth: usize,
) -> Result<(), ConnectorError> {
    for (key, nested) in table {
        let key: String = key
            .chars()
            .filter(char::is_ascii_alphanumeric)
            .flat_map(char::to_lowercase)
            .collect();
        if secret_key(&key) {
            return Err(ConnectorError::new(
                "config",
                "embedded credential fields are forbidden; authentication belongs to the provider",
            ));
        }
        reject_value(nested, depth + 1)?;
    }
    Ok(())
}

fn secret_key(value: &str) -> bool {
    matches!(
        value,
        "password"
            | "token"
            | "accesstoken"
            | "refreshtoken"
            | "clientsecret"
            | "privatekey"
            | "apikey"
            | "credential"
            | "credentials"
            | "secret"
    )
}
