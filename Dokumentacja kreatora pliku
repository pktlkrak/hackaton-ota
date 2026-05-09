# main 
Tworzy i zapisuje na dysku klucz prywatny oraz woła odpowienie fukcje.
Jeśli nie uda stworzyć się kluczy pliki zostaną usunięte.

# parse_semver 
```rust
str->Result<Semver>
```

funkcja parsuje string do Semver lub extended Semver w zależności od wysłanych danych 

# package_file
**Funkcja najpierw usuwa dane z istniejących plików docelowych, potem je zapisuje**

```rust
private_key_file: String, version: String, installer_path: String,output: String -> Result<()>
```

Sprawdza czy podany plik klucza ma odpownienią długość (*40*). 
Wyłskuje z metadanych podaną długość danych.
Tworzy kp oraz id z klucza prywatnego, po czym tworzy klucz MlDsa87.
```
AdditionalMetadata = { length: SECOND_STAGE_OFFSET + installer_length, semver}
```
Po utworzeniu pliku aktualizacji tworzy hash SHA512 i podpisuje nim plik aktualizacji.


```rust
output.seek(std::io::SeekFrom::Start(0))?;
output.write_all("UPXD0001".as_bytes())?;
output.write_all(&id.to_le_bytes())?;
output.write_all(&shasum)?;
```
