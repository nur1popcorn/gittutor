# gittutor

gittutor is a small command line game designed to help you improve your usage of git.
The tool generates a score for your commits based on various subjectively nice to follow rules such as the formatting of the commit message.

When gittutor is executed on a local git repository it produces a top 10 list of all the best git user that contributed to the repository.
Below an example of this can be seen on the LibAFL git repo:
```
$ gittutor
#1      (21911) Andrea Fioraldi andreafioraldi@gmail.com
#2      (12481) Dominik Maier domenukk@gmail.com
#3      (3706)  Dongjia Zhang tokazerkje@outlook.com
#4      (3086)  s1341 s1341@users.noreply.github.com
#5      (2351)  Dominik Maier dmnk@google.com
#6      (2235)  Toka tokazerkje@outlook.com
#7      (1545)  julihoh julihoh@users.noreply.github.com
#8      (1053)  David CARLIER devnexen@gmail.com
#9      (753)   Dongjia "toka" Zhang tokazerkje@outlook.com
#10     (604)   syheliel 45957390+syheliel@users.noreply.github.com
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
