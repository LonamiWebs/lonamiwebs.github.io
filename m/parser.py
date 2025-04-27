def parse_toml_ish(content: bytes) -> dict[bytes, list[bytes]]:
    result: dict[bytes, list[bytes]] = {}

    for line in content.splitlines():
        line = line.strip()
        if not line or line.startswith(b"["):
            continue
        name, value = map(bytes.strip, line.split(b"=", maxsplit=1))
        values = (
            [v.strip(b'[" ]') for v in value.split(b",")]
            if value.startswith(b"[")
            else [value.strip(b'" ')]
        )
        result[name.strip(b'" ')] = values

    return result
