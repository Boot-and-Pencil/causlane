package verification.fixtures.alloy;

import edu.mit.csail.sdg.alloy4.A4Reporter;
import edu.mit.csail.sdg.ast.Command;
import edu.mit.csail.sdg.parser.CompModule;
import edu.mit.csail.sdg.parser.CompUtil;
import edu.mit.csail.sdg.translator.A4Options;
import edu.mit.csail.sdg.translator.A4Solution;
import edu.mit.csail.sdg.translator.TranslateAlloyToKodkod;
import java.util.logging.ConsoleHandler;
import java.util.logging.Formatter;
import java.util.logging.Level;
import java.util.logging.LogRecord;
import java.util.logging.Logger;

public final class AlloyRunner {
    private static final Logger LOG = createLogger();

    private static Logger createLogger() {
        Logger logger = Logger.getLogger(AlloyRunner.class.getName());
        logger.setUseParentHandlers(false);
        ConsoleHandler handler = new ConsoleHandler();
        handler.setFormatter(new Formatter() {
            @Override
            public String format(LogRecord logRecord) {
                return formatMessage(logRecord) + System.lineSeparator();
            }
        });
        logger.addHandler(handler);
        return logger;
    }

    public static void main(String[] args) throws Exception {
        if (args.length != 1) {
            LOG.severe("usage: AlloyRunner <model.als>");
            System.exit(2);
        }
        A4Reporter reporter = new A4Reporter();
        CompModule world = CompUtil.parseEverything_fromFile(reporter, null, args[0]);
        A4Options options = new A4Options();
        options.solver = kodkod.engine.satlab.SATFactory.DEFAULT;
        boolean holds = true;
        for (Command command : world.getAllCommands()) {
            A4Solution solution = TranslateAlloyToKodkod.execute_command(
                reporter, world.getAllReachableSigs(), command, options
            );
            boolean commandHolds = command.check
                ? !solution.satisfiable()
                : solution.satisfiable();
            LOG.log(Level.INFO, "{0}={1}", new Object[] {command.label, commandHolds});
            holds &= commandHolds;
        }
        System.exit(holds ? 0 : 1);
    }
}

