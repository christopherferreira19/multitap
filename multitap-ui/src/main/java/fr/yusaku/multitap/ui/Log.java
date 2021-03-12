package fr.yusaku.multitap.ui;

import java.io.PrintStream;
import java.io.PrintWriter;
import java.io.StringWriter;
import java.util.logging.*;

public class Log {

    private static final Logger logger;

    static {
        logger = get(Log.class.getPackage());
        logger.setUseParentHandlers(false);
        Handler handler = new ConsoleHandler();
        handler.setLevel(Level.INFO);
        handler.setFormatter(new OneLinerLoggerFormatter());
        logger.addHandler(handler);
    }

    public static Logger get(Package ppackage) {
        return get(ppackage.getName());
    }

    public static Logger get(Class<?> clazz) {
        return get(clazz.getCanonicalName());
    }

    public static Logger get(String name) {
        if (name.startsWith(Log.class.getPackageName())) {
            name = name.replace(Log.class.getPackageName(), "multitap_ui");
        }
        return Logger.getLogger(name);
    }

    public static class OneLinerLoggerFormatter extends Formatter {

        public OneLinerLoggerFormatter() {
        }

        @Override
        public String format(LogRecord record) {
            StringBuilder builder = new StringBuilder();
            builder.append("[").append(record.getLevel().getName().toUpperCase()).append("]");
            builder.append("[").append(record.getLoggerName()).append("]");
            builder.append(" ").append(formatMessage(record));
            builder.append("\n");

            Throwable thrown = record.getThrown();
            if (thrown != null) {
                try {
                    StringWriter sw = new StringWriter();
                    PrintWriter pw = new PrintWriter(sw);
                    thrown.printStackTrace(pw);
                    pw.close();
                    builder.append(sw.toString());
                }
                catch (Exception ex) {
                    // Not much we can do
                }
            }

            return builder.toString();
        }
    }

    public static class SystemLoggerHandler extends Handler {

        private final PrintStream err;

        public SystemLoggerHandler() {
            this.err = System.err;
            setFormatter(new OneLinerLoggerFormatter());
        }

        @Override
        public void publish(LogRecord record) {
            if (record.getLevel().intValue() < getLevel().intValue()) {
                return;
            }

            err.print(getFormatter().format(record));
        }

        @Override
        public void flush() {
            err.flush();
        }

        @Override
        public void close() throws SecurityException {
            flush();
        }
    }
}

