#!/bin/bash
set -xeu

add_mutation_annotations() {
    set +x
    if [ -f "$1" ]; then
        echo "Add mutation annotation in $1..."
        replace=1
        while IFS= read -r line; do
            if [[ "$line" == "#[cfg(test)]" ]] || [[ "$line" =~ .*"coverage: off".* ]]; then
                replace=0
            elif [[ "$line" =~ .*"coverage: on".* ]]; then
                replace=1
            fi
            if [[ "$line" =~ ^[^/]+.*" fn ".* ]] || [[ "$line" =~ ^"fn ".* ]] && [ $replace -eq 1 ]; then
                echo "#[::mutagen::mutate] $line" >> "$1.tmp"
            else
                echo "$line" >> "$1.tmp"
            fi
        done < "$1"

        # Fix to avoid "annotation needed" due to a bug with serde_json (https://github.com/llogiq/mutagen/issues/164)
        sed -e "s/assert_eq!(\(.*\), &\[\])/assert!(\1.is_empty())/g" -i "$1.tmp"
        sed -e "s/assert_eq!(\(.*\), \[\])/assert!(\1.is_empty())/g" -i "$1.tmp"
        sed -e "s/assert_eq!(\(.*\), Vec::new())/assert!(\1.is_empty())/g" -i "$1.tmp"
        sed -e "s/assert_eq!(\(.*\), vec!\[\])/assert!(\1.is_empty())/g" -i "$1.tmp"

        rm "$1"
        mv "$1.tmp" "$1"
    fi
    set -x
}

git clone https://github.com/llogiq/mutagen
cd mutagen
git checkout "$MUTAGEN_COMMIT"
cd ..
cargo install --path ./mutagen/mutagen-runner --debug

IFS=";"
for crate_path in ./crates/*; do
    echo "$NOT_MUTATED_CRATES" | grep -E "(^| )$(basename "$crate_path")($| )" && continue
    cd "$crate_path"
    while IFS= read -r -d '' file; do
        add_mutation_annotations "$file"
    done< <(find ./src -type f -name '*.rs' -print0)
    echo "Annotations added."
    echo "Add mutagen dependency in Cargo.toml..."
    sed -i -re "s;(\[dependencies\]);\1\nmutagen = { git = \"https://github.com/llogiq/mutagen\", rev = \"$MUTAGEN_COMMIT\" };" Cargo.toml
    echo "Dependency added."

    cargo test --no-run --verbose
    cargo-mutagen | tee log.txt

    sed -i "/mutagen.*/d" Cargo.toml
    while IFS= read -r -d '' file; do
        sed -i "s;#\[::mutagen::mutate\] ;;" "$file"
    done< <(find ./src -type f -name '*.rs' -print0)
    cd -
done

for crate_path in ./crates/*; do
    echo "$NOT_MUTATED_CRATES" | grep -E "(^| )$(basename "$crate_path")($| )" && continue
    cd "$crate_path"
    killed=$(grep -o '([^"]*%) mutants killed' log.txt | grep -o '[0-9.]*')
    rm log.txt
    failure=$(awk 'BEGIN{ print '"$killed"'<'"$MUTAGEN_THRESHOLD"' }')
    if [ "$failure" -eq "1" ]; then
        echo "Mutation tests have failed with $killed% of killed mutants instead of at least $MUTAGEN_THRESHOLD%."
        exit 1
    fi
    cd -
done
