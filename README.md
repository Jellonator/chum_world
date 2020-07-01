# Chum World
*THIS TOOL IS GOING TO BE DEPRECATED. DEVELOPMENT IS MORE ACTIVE AT https://github.com/Jellonator/chum-world/*

This is a tool used to edit Revenge of the Flying Dutchman .DGC/.NGC archives. There are two ways to use this tool: command-line mode, and GUI mode. This README will cover both.

## GUI mode
By double clicking on the program, or by running the program with no arguments, the program can be started in GUI mode.  GUI mode allows for editing .DGC/.NGC archives with a graphical interface that isn't too dissimilar to an archive manager or a file manager. When opening the program, you will be greeted by an interface that looks something like this:

![No Open Files](https://raw.githubusercontent.com/Jellonator/chum_world/master/images/scrn1.png)

### Opening files
In order to open a file, click the "open" button on the top right, then select the .DGC file that you want to edit. A .DGC file must have a matching .NGC file of the same name in the same directory. You should see something like the below image after opening an archive:

![Open File](https://raw.githubusercontent.com/Jellonator/chum_world/master/images/scrn2.png)

Editing files:
To edit a file in a DGC archive, you can click on its name on the left. You will be greeted by a new pane that opens on the right that allows you to edit the selected file:

![Open File](https://raw.githubusercontent.com/Jellonator/chum_world/master/images/scrn3.png)

You can change a file's name by editing the 'Name' entry. Likewise, you can edit a file's type and subtype in the same manner. All files have a type, but some files do not have a sub-type.

The editor pane provides a default editor for specific file types. The text editor, for example, can be used to edit TXT files. For example, I changed some text in the image below:

![Open File](https://raw.githubusercontent.com/Jellonator/chum_world/master/images/scrn4.png)

For files that do not have an editor, the file can instead be extracted from the archive from the 'Extract' button. After editing the extracted file with an external editor, you can then re-import that file back into the archive using the 'Replace' button. The below example shows a file that does have a sub-type, but it's type does not have an existing editor for it (at least not yet).

![Open File](https://raw.githubusercontent.com/Jellonator/chum_world/master/images/scrn5.png)

### Saving files
Once you are done editing an archive, you can click the 'Save' button on the upper-right to save the archive. You can also use the drop-down menu and select 'Save as' to save the archive to a different location. Be sure to save and backup your changes often.

## Command line mode
You can get a list of available commands using `chum_world help`, which will print out this:

```
Chum World 0.2.0
James "Jellonator" B. <jellonator00@gmail.com>
Edits Revenge of the Flying Dutchman archive files

USAGE:
    chum_world [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    extract    Extract the contents of an archive to a json file as well as a folder
    help       Prints this message or the help of the given subcommand(s)
    info       Get information about the given archive
    list       Lists the contents of the given archive
    pack       Pack the extracted contents of an archive back into an archive
```

The command names are fairly self-explanatory. Use `chum_world help {command}` for more informaiton about the given command.

## Compiling
You will need the Rust compiler to compile this program. You can get it here: https://www.rust-lang.org/en-US/install.html.

To compile this program, run `cargo build --release` in the root directory of this repository. You may need to install gtk3 development packages.

Once the program has been compiled, you can find it in `./target/release/chum_world`.
