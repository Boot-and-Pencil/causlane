// Headless Alloy runner for causlane formal checks.
//
// Loads an .als model, executes every command (check/run) with the bundled
// SAT4J solver, and prints a one-line JSON summary. Exit code 0 iff every
// command "holds" (a `check` holds when UNSAT = no counterexample; a `run`
// holds when SAT = an instance exists).
//
// Build: javac -cp .tools/alloy/alloy.jar -d .tools/alloy/classes formal/tools/AlloyRunner.java
// Run:   java -cp .tools/alloy/alloy.jar:.tools/alloy/classes AlloyRunner <model.als>

import edu.mit.csail.sdg.alloy4.A4Reporter;
import edu.mit.csail.sdg.ast.Command;
import edu.mit.csail.sdg.parser.CompModule;
import edu.mit.csail.sdg.parser.CompUtil;
import edu.mit.csail.sdg.translator.A4Options;
import edu.mit.csail.sdg.translator.A4Solution;
import edu.mit.csail.sdg.translator.TranslateAlloyToKodkod;

public final class AlloyRunner {
    public static void main(String[] args) throws Exception {
        if (args.length < 1) {
            System.err.println("usage: AlloyRunner <model.als>");
            System.exit(2);
            return;
        }
        String file = args[0];
        A4Reporter rep = new A4Reporter();
        CompModule world = CompUtil.parseEverything_fromFile(rep, null, file);
        A4Options opt = new A4Options();
        opt.solver = kodkod.engine.satlab.SATFactory.DEFAULT;

        StringBuilder sb = new StringBuilder();
        sb.append("{\"model\":\"").append(esc(file)).append("\",\"commands\":[");
        boolean allOk = true;
        boolean first = true;
        for (Command cmd : world.getAllCommands()) {
            A4Solution sol =
                TranslateAlloyToKodkod.execute_command(rep, world.getAllReachableSigs(), cmd, opt);
            boolean sat = sol.satisfiable();
            boolean holds = cmd.check ? !sat : sat;
            if (!holds) {
                allOk = false;
            }
            if (!first) {
                sb.append(",");
            }
            first = false;
            sb.append("{\"command\":\"").append(esc(cmd.label)).append("\",")
              .append("\"kind\":\"").append(cmd.check ? "check" : "run").append("\",")
              .append("\"satisfiable\":").append(sat).append(",")
              .append("\"holds\":").append(holds).append("}");
        }
        sb.append("],\"status\":\"").append(allOk ? "pass" : "fail").append("\"}");
        System.out.println(sb.toString());
        System.exit(allOk ? 0 : 1);
    }

    private static String esc(String s) {
        if (s == null) {
            return "";
        }
        return s.replace("\\", "\\\\").replace("\"", "\\\"");
    }
}
