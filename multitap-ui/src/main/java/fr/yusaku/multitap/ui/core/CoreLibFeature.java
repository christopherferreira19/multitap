package fr.yusaku.multitap.ui.core;

import com.oracle.svm.core.jni.JNIRuntimeAccess;
import com.oracle.svm.core.util.UserError;
import com.oracle.svm.hosted.FeatureImpl;
import com.oracle.svm.hosted.ImageClassLoader;
import org.graalvm.nativeimage.ImageSingletons;
import org.graalvm.nativeimage.Platform;
import org.graalvm.nativeimage.Platforms;
import org.graalvm.nativeimage.hosted.Feature;

import java.io.IOException;
import java.lang.reflect.Method;
import java.nio.file.Files;
import java.nio.file.Path;

import static java.nio.file.StandardCopyOption.COPY_ATTRIBUTES;
import static java.nio.file.StandardCopyOption.REPLACE_EXISTING;

class CoreLibFeature implements Feature {

    public static final String LIB_NAME = "multitap_ui_core";

    @Platforms(Platform.HOSTED_ONLY.class)
    private static class Build {

        private enum Profile {
            Debug("debug"),
            Release("release"),
            ;

            private final String dir;

            Profile(String dir) {
                this.dir = dir;
            }

            static Profile fromProperty() {
                var corelibArgs = System.getProperty("maven.corelib.args");
                return corelibArgs.contains("--release") ? Release : Debug;
            }
        }

        private enum Target {
            Host(null, "x86_64-linux"),
            Android("aarch64-linux-android", "aarch64-android"),
            ;

            private final String cargoDir;
            private final String gluonDir;

            Target(String cargoDir, String gluonDir) {
                this.cargoDir = cargoDir;
                this.gluonDir = gluonDir;
            }

            static Target fromProperty() {
                var property = System.getProperty("maven.client.target");
                switch (property) {
                    case "host": return Host;
                    case "android": return Android;
                    default: throw UserError.abort("Unknown `client.target` property: %s, expected \"host\" or \"android\"", property);
                }
            }
        }

        private final Profile profile;
        private final Target target;
        private final Path multitapRoot;
        private final Path buildDir;

        private Build() {
            this.profile = Profile.fromProperty();
            this.target = Target.fromProperty();
            this.multitapRoot = Path.of(System.getProperty("maven.multitap.basedir"));
            this.buildDir = Path.of(System.getProperty("maven.project.build.directory"));
        }

        private static void registerJNICallbackFromNative(FeatureImpl.BeforeAnalysisAccessImpl access) {
            ImageClassLoader classLoader = access.getImageClassLoader();
            for (Method method : classLoader.findAnnotatedMethods(JNICallback.class)) {
                JNIRuntimeAccess.register(method.getDeclaringClass());
                JNIRuntimeAccess.register(method);
            }
        }

        public Path corelibSrcDir() {
            var path = multitapRoot;
            path = path.resolve("target");
            if (target.cargoDir != null) {
                path = path.resolve(target.cargoDir);
            }
            path = path.resolve(profile.dir);
            return path;
        }

        public Path corelibDstDir() {
            return buildDir.resolve("client").resolve(target.gluonDir).resolve("gvm/lib");
        }

        private void copyStaticLibForLinker() throws IOException {
            var libFilename = "lib" + LIB_NAME + ".a";
            var srcLib = corelibSrcDir().resolve(libFilename);
            var dstLib = corelibDstDir().resolve(libFilename);

            System.out.println("Copying library " + LIB_NAME + ", from " + srcLib.getParent() + " to " + dstLib.getParent());

            if (!dstLib.getParent().toFile().exists() && !dstLib.getParent().toFile().mkdirs()) {
                System.out.println("Unable to create directory for static lib: " + dstLib.getParent());
            }
            else {
                Files.copy(srcLib, dstLib, REPLACE_EXISTING, COPY_ATTRIBUTES);
            }
        }
    }

    public static void loadLibrary() {
        if (!ImageSingletons.contains(CoreLibFeature.class)) {
            System.loadLibrary(CoreLibFeature.LIB_NAME);
        }
    }

    @Override
    public void beforeAnalysis(BeforeAnalysisAccess access) {
        Build.registerJNICallbackFromNative((FeatureImpl.BeforeAnalysisAccessImpl) access);
    }

    @Override
    public void afterCompilation(AfterCompilationAccess access) {
        try {
            new Build().copyStaticLibForLinker();
        }
        catch (IOException e) {
            System.out.println("Unable to copy library " + LIB_NAME);
            e.printStackTrace();
        }
    }
}
