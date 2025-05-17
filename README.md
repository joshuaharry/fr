# fr

An extremely simple find-replace tool.

## Usage

If you type:

```bash
fr "find_this_text" "replace_with_that_text" 
```

Then `fr` will, starting with the current working directory, walk through
the file tree and replace all of the text `find_this_text` with the replacement
you have entered. It will match the text literally, without using regular
expressions.

If you are working in a git repository, `fr` will use the `.gitignore` file 
at the root of your project and ignore any files that match it. `fr` will also
ignore binary files automatically.

## Installing

Download one of the binaries from the release; put it on your `$PATH`.

## Inspiration

In the past, I've used [fastmod](https://github.com/facebookincubator/fastmod?tab=readme-ov-file)
and [this StackExchange thread](https://superuser.com/questions/428493/how-can-i-do-a-recursive-find-and-replace-from-the-command-line) to solve
this problem. They might be sufficient for you.