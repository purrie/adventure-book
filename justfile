export VERSION := `cargo metadata --no-deps -q --format-version=1 | grep -Eo '"version":"[0-9].[0-9].[0-9]"' | grep -Eo '[0-9].[0-9].[0-9]'`

build-linux:
    cargo build --release --target x86_64-unknown-linux-gnu

build-windows:
    cargo rustc --release --target x86_64-pc-windows-gnu -- -Clink-args="-Wl,--subsystem,windows"

[linux]
install: build-linux
    cp ./target/x86_64-unknown-linux-gnu/release/adventure-book ~/.local/bin/
    mkdir -p ~/.local/share/adventure-book/
    cp -r ./data/* ~/.local/share/adventure-book/
    @echo Installation complete

[linux]
remove:
    rm ~/.local/bin/adventure-book
    rm -r ~/.local/share/adventure-book/help
    rm -r ~/.local/share/adventure-book/images
    @echo Removal complete.
    @echo You may want to call purge to remove installed user adventures and data

[linux]
purge:
    #!/usr/bin/env bash
    if [ -f ~/.local/bin/adventure-book ]; then
        rm ~/.local/bin/adventure-book
    fi
    rm -rf ~/.local/share/adventure-book
    echo Purge complete

pack-all: pack-zip pack-tar pack-deb
    @echo Packing everything completed

pack-zip: build-windows
    #!/usr/bin/env bash
    mkdir -p ./target/pack/Adventure\ Book
    cp ./target/x86_64-pc-windows-gnu/release/adventure-book.exe ./target/pack/Adventure\ Book/Adventure\ Book.exe
    cp -r ./data ./target/pack/Adventure\ Book/data
    cd target/pack/
    rm Adventure\ Book.zip
    zip -r Adventure\ Book Adventure\ Book
    rm -r Adventure\ Book/
    echo Packing Zip Complete

pack-tar: build-linux
    #!/usr/bin/env bash
    mkdir -p ./target/pack/adventure-book
    cp ./target/x86_64-unknown-linux-gnu/release/adventure-book ./target/pack/adventure-book
    cp -r ./data ./target/pack/adventure-book/data
    cd target/pack/adventure-book
    echo '#!/usr/bin/bash' >> install.sh
    echo 'if [ -z ${DESTDIR+unset} ]; then' >> install.sh
    echo '    echo Automatically determining installation path, set DESTDIR variable to determine where to put the executable file' >> install.sh
    echo '    echo $PATH | grep "$HOME/.local/bin" > /dev/null 2>/dev/null' >> install.sh
    echo '    if [ $? -eq 0 ]; then' >> install.sh
    echo '        echo Installing to .local/bin' >> install.sh
    echo '        cp ./adventure-book ~/.local/bin/' >> install.sh
    echo '    else' >> install.sh
    echo '        echo $PATH | grep "$HOME/bin" > /dev/null 2>/dev/null' >> install.sh
    echo '        if [ $? -eq 0 ]; then' >> install.sh
    echo '            echo Installing to bin' >> install.sh
    echo '            cp ./adventure-book ~/bin/' >> install.sh
    echo '        else' >> install.sh
    echo '            echo $PATH | grep "$HOME/.bin" > /dev/null 2>/dev/null' >> install.sh
    echo '            if [ $? -eq 0 ]; then' >> install.sh
    echo '                echo Installing to .bin' >> install.sh
    echo '                cp ./adventure-book ~/.bin/' >> install.sh
    echo '            else' >> install.sh
    echo '                echo Could not automatically detect any path in your HOME directory. Ensure to add either $HOME/.local/bin $HOME/.bin or $HOME/bin to your PATH variable' >> install.sh
    echo '                exit 1' >> install.sh
    echo '            fi' >> install.sh
    echo '        fi' >> install.sh
    echo '    fi' >> install.sh
    echo 'else' >> install.sh
    echo '    echo Installing to $DESTDIR' >> install.sh
    echo '    cp ./adventure-book $DESTDIR' >> install.sh
    echo 'fi' >> install.sh
    echo 'echo Installing data to .local/share/adventure-book' >> install.sh
    echo "mkdir -p ~/.local/share/adventure-book" >> install.sh
    echo "cp -r ./data/* ~/.local/share/adventure-book/" >> install.sh
    echo "echo Installation completed for local user" >> install.sh
    chmod 755 install.sh
    echo '#!/usr/bin/bash' >> remove.sh
    echo 'AB=$(which adventure-book)' >> remove.sh
    echo 'if [ $? -eq 0 ]; then' >> remove.sh
    echo '    echo Removed executable' >> remove.sh
    echo '    rm $AB' >> remove.sh
    echo 'fi' >> remove.sh
    cp remove.sh purge.sh
    echo 'if [ -d $HOME/.local/share/adventure-book/images ]; then' >> remove.sh
    echo '    rm -rf $HOME/.local/share/adventure-book/images' >> remove.sh
    echo '    echo Removed images' >> remove.sh
    echo 'fi' >> remove.sh
    echo 'if [ -d $HOME/.local/share/adventure-book/help ]; then' >> remove.sh
    echo '    rm -rf $HOME/.local/share/adventure-book/help' >> remove.sh
    echo '    echo Removed help articles' >> remove.sh
    echo 'fi' >> remove.sh
    echo 'echo Removal complete, installed books have been preserved, use purge.sh to remove everything.' >> remove.sh
    chmod 755 remove.sh
    echo 'if [ -d $HOME/.local/share/adventure-book ]; then' >> purge.sh
    echo '    rm -rf $HOME/.local/share/adventure-book' >> purge.sh
    echo '    echo Purged the data folder' >> purge.sh
    echo 'fi' >> purge.sh
    chmod 755 purge.sh
    cd ..
    tar -cf ./adventure-book.tar.gz ./adventure-book
    rm -r adventure-book/
    echo Packing Tar Complete

pack-deb: build-linux
    #!/usr/bin/env bash
    VERSION_MAJOR=$(echo $VERSION | sed 's/.[0-9]$//')
    VERSION_MINOR=$(echo $VERSION | sed 's/[0-9].[0-9].//')
    TARGET_FOLDER=adventure-book_$VERSION_MAJOR-$VERSION_MINOR

    mkdir -p ./target/pack/$TARGET_FOLDER/usr/bin
    mkdir -p ./target/pack/$TARGET_FOLDER/usr/share/adventure-book

    cp ./target/x86_64-unknown-linux-gnu/release/adventure-book ./target/pack/$TARGET_FOLDER/usr/bin/
    cp -r ./data/* ./target/pack/$TARGET_FOLDER/usr/share/adventure-book
    cd ./target/pack/$TARGET_FOLDER
    mkdir DEBIAN
    cd DEBIAN
    echo Package: adventure-book >> control
    echo Version: $VERSION_MAJOR-$VERSION_MINOR >> control
    echo Section: games >> control
    echo Priority: optional >> control
    echo Architecture: amd64 >> control
    echo "Maintainer: Purrie Brightstar <purriestarshine@gmail.com>" >> control
    echo Description: A choose your own adventure text game >> control

    cp ../../../../LICENSE ./copyright
    if [ -f ../../changelog ]; then
        cp ../../changelog ./changelog
    else
        echo No changelog found in root/target/pack folder, skipping inclusion
    fi

    cd ../..
    dpkg-deb --build $TARGET_FOLDER
    rm -r $TARGET_FOLDER
    echo Packing Deb Complete
