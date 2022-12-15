# gittutor

gittutor is a small command line game designed to help you improve your usage of git.
The tool generates a score for your commits based on various subjectively nice to follow rules such as the formatting of the commit message.

When gittutor is executed on a local git repository it produces a top 10 list of all the best git user that contributed to the repository.
Below an example of this can be seen on the 
```
$ gittutor

```

The program can also produce a plot which helps you visualize where you lost points inorder to improve your score:
```
$ gittutor --author 
```

For more usages look at `gittutor --help`.

## Build

```
cargo build -r
```
