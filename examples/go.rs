use genco::fmt;
use genco::prelude::*;

fn main() -> anyhow::Result<()> {
    let cdktf = &go::import("github.com/hashicop/terraform-cdk-go/cdktf", "");
    let constructs = &go::import("github.com/aws/constructs-go/constructs/v10", "constructs");
    let jsii = &go::import("github.com/aws/jsii-runtime-go", "jsii");
    let googleprovider = &go::import(
        "github.com/cdktf/cdktf-provider-google-go/google/v13/provider",
        "",
    );

    let tokens = quote! {
        func NewMyStack(scope $constructs.Construct, id string) cdktf.TerraformStack {
            stack := $cdktf.NewTerraformStack(scope, &id)

            googleprovider.NewGoogleProvider(stack, $jsii.String("google"), &$googleprovider.GoogleProviderConfig{})
        }

        func main() {
            app := $cdktf.NewApp(nil)

            NewMyStack(app, "my-stack")

            app.Synth()
        }
    };

    let stdout = std::io::stdout();
    let mut w = fmt::IoWriter::new(stdout.lock());

    let fmt = fmt::Config::from_lang::<Go>();
    let config = go::Config::default().with_package("main");

    tokens.format_file(&mut w.as_formatter(&fmt), &config)?;
    Ok(())
}
