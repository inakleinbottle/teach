use std::fmt;
use std::fs;
use std::io::prelude::*;
use std::path::Path;



use crate::TeachResult;


pub struct MakeTarget<'a, T, P, R> 
    where T: AsRef<str>,
          P: AsRef<str>,
          R: AsRef<str>
{
    targets: &'a [T],
    prereqs: &'a [P],
    recipe: &'a [R]
}

impl<'a, T, P, R> fmt::Display for MakeTarget<'a, T, P, R>  
    where T: AsRef<str>,
          P: AsRef<str>,
          R: AsRef<str>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut spacer = "";
        for target in self.targets.iter() {
            write!(f, "{}{}", spacer, target.as_ref())?;
            spacer = " ";
        }
        write!(f, ":")?;
        
        for prereq in self.prereqs.iter() {
            write!(f, " {}", prereq.as_ref())?;
        }

        write!(f, "\n")?;

        for line in self.recipe.iter() {
            writeln!(f, "\t{}", line.as_ref())?;
        }

        Ok(())
    }
}



pub struct Makefile<'a, 'b: 'a, V, T, P, R> 
    where V: AsRef<str>,
          T: AsRef<str>,
          P: AsRef<str>,
          R: AsRef<str>
{
    vars: &'a [V],
    rules: &'a [MakeTarget<'b, T, P, R>]
}

impl<'a, 'b: 'a, V, T, P, R> fmt::Display for Makefile<'a, 'b, V, T, P, R> 
    where V: AsRef<str>,
          T: AsRef<str>,
          P: AsRef<str>,
          R: AsRef<str>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { 
        for var in self.vars.iter() {
            writeln!(f, "{}", var.as_ref())?;
        }
        writeln!(f, "\n")?;

        for rule in self.rules.iter() {
            writeln!(f, "{}", rule)?;
        }


        Ok(())
    }
}




pub fn write_sheet_makefile<S: AsRef<str>>(
    name: &str,
    root: &Path,
    problems: &[S],
) -> TeachResult<()> {

    let problem_rule = MakeTarget {
        targets: &[format!("{}-problems.pdf", name)],
        prereqs: &[format!("{}-problems.tex", name), 
                   String::from("$(PROBLEMS)")],
        recipe: &["$(TEX) $(TEXFLAGS) $<", 
                  "$(TEX) $(TEXFLAGS) $<",
                  "$(RM) *.log *.aux"]
    };

    let solution_rule = MakeTarget {
        targets: &[format!("{}-solutions.pdf", name)],
        prereqs: &[format!("{}-solutions.tex", name), 
                   String::from("$(SOLUTIONS)")],
        recipe: &["$(TEX) $(TEXFLAGS) $<", 
                  "$(TEX) $(TEXFLAGS) $<",
                  "$(RM) *.log *.aux"]
    };

    let mut probs_var = String::from("PROBS = $(addprefix $(PROBDIR)/,");
    for prob in problems.iter() {
        probs_var.push(' ');
        probs_var.push_str(prob.as_ref());
    }
    probs_var.push(')');


    let makefile = Makefile {
        vars: &[
            probs_var.as_str(),
            "PROBLEMS = $(addsuffix /problem.tex, $(PROBS))",
            "SOLUTIONS = $(addsuffix /solution.tex, $(PROBS))"
        ],
        rules: &[
            problem_rule,
            solution_rule
        ]
    };

    fs::write(root.join(format!("{}.mk", name)), makefile.to_string())?;

    Ok(())
}

pub fn write_component_makefile(
    path: &Path,
    problems_dir: &str,
    include_dirs: &[&str],

) -> TeachResult<()> {
    println!("Creating makefile: {}", path.display());

    let probdir =  format!("PROBDIR=../{}", problems_dir);
    let mut TEXINPUTS = String::from("export TEXINPUTS=../");
    TEXINPUTS.push_str(problems_dir);
    include_dirs.iter().for_each(|d| {
        TEXINPUTS.push(':');
        TEXINPUTS.push_str(d);
    });
    TEXINPUTS.push(':');


    let vars = &[
        "TEX = pdflatex",
        "TEXFLAGS = -interaction=nonstopmode",
        "DIRS = $(wildcard */.)",
        "PDF_FILES = $(notdir $(patsubst %.tex, %.pdf, $(wildcard */*.tex)))",
        probdir.as_str(),
        "vpath %.tex $(DIRS)",
        TEXINPUTS.as_str(),
    ];
    let mf = Makefile {
        vars: vars,
        rules: &[
            MakeTarget {
                targets: &[".PHONY"],
                prereqs: &["all"],
                recipe: &[""],
            },
            MakeTarget {
                targets: &["all"],
                prereqs: &["$(PDF_FILES)"],
                recipe: &[""],
            }
        ]
    };

    fs::write(path.join("Makefile"), format!("{}\n\ninclude */*.mk", mf))?;

    Ok(())
}





#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_make_rule() {
        let mr = MakeTarget {
            targets: &["target"],
            prereqs: &["prereq"],
            recipe: &["@echo test"],
        };

        let expected = "target: prereq\n\t@echo test\n";

        assert_eq!(mr.to_string(), expected);

    }
}