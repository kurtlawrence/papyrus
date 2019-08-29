#[macro_use]
extern crate criterion;

use colored::Colorize;

use criterion::Criterion;

use papyrus::prelude::code::{Input, SourceCode, Statement, StmtGrp};
use papyrus::prelude::*;

fn pfh_compile_construct(c: &mut Criterion) {
    use papyrus::prelude::code::construct_source_code;

    let linking = papyrus::prelude::linking::LinkingConfiguration::default();
    let map = vec![
        ("lib".into(), src_code()),
        ("test".into(), src_code()),
        ("test/inner".into(), src_code()),
        ("test/inner/deep".into(), src_code()),
    ]
    .into_iter()
    .collect();

    c.bench_function("construct_source_code", move |b| {
        b.iter(|| construct_source_code(&map, &linking))
    });
}

fn rustfmt(c: &mut Criterion) {
    let code = "let a = 1 ; let b = 2 ; a + b ";
    c.bench_function("format code", move |b| {
        b.iter(|| papyrus::fmt::format(code))
    });
}

criterion_group!(benches, pfh_compile_construct, rustfmt);
criterion_main!(benches);

fn cstr() -> String {
    format!(
        "{}{}{}{}{}{}{}{}{}{}{}",
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit.".red(),
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit.".blue(),
        "Donec vel metus nec nisl ultrices cursus.".green(),
        "In in enim eget felis elementum consectetur et nec nisi.".purple(),
        "Morbi vel sapien consectetur, tristique sem id, facilisis purus.".yellow(),
        "Vivamus bibendum nisi ac lacus euismod hendrerit vel ac lacus.".red(),
        " Nulla scelerisque ipsum eu lacus dignissim, a tempus arcu egestas.".white(),
        "Nulla scelerisque ipsum eu lacus dignissim, a tempus arcu egestas.".bright_red(),
        "Praesent lobortis quam sed erat egestas, et tincidunt erat rutrum.".bright_white(),
        "Nullam maximus mauris a ultricies blandit.".bright_green(),
        "Morbi eget neque eget neque viverra mollis in id lacus.".bright_purple(),
    )
}

fn src_code() -> SourceCode {
    SourceCode {
        items: vec![],
        crates: vec![],
        stmts: vec![
            StmtGrp(vec![Statement {
                expr: LOREM_IPSUM.to_string(),
                semi: false,
            }]),
            StmtGrp(vec![Statement {
                expr: LOREM_IPSUM.to_string(),
                semi: true,
            }]),
            StmtGrp(vec![Statement {
                expr: LOREM_IPSUM.to_string(),
                semi: false,
            }]),
        ],
    }
}

const LOREM_IPSUM: &str = r#"
    Lorem ipsum dolor sit amet, consectetur adipiscing elit.
    Donec vel metus nec nisl ultrices cursus.

    In in enim eget felis elementum consectetur et nec nisi.
    Morbi vel sapien consectetur, tristique sem id, facilisis purus.

    Vivamus bibendum nisi ac lacus euismod hendrerit vel ac lacus.
    Nulla scelerisque ipsum eu lacus dignissim, a tempus arcu egestas.

    Praesent lobortis quam sed erat egestas, et tincidunt erat rutrum.
    Nullam maximus mauris a ultricies blandit.
    Morbi eget neque eget neque viverra mollis in id lacus.

    Curabitur vitae neque auctor orci maximus ornare.
    Nullam eleifend lacus vitae nulla consectetur laoreet sed vel lorem.
    Cras venenatis felis a fringilla pretium.

    Nulla nec nulla a velit condimentum dapibus.
    Quisque mollis nisl pretium urna rutrum tincidunt.

    Suspendisse vel tellus viverra, ornare tortor a, aliquam dolor.
    Nam molestie elit quis tempus cursus.
    Maecenas luctus enim id purus maximus, nec commodo enim lobortis.

    Donec tempus quam vitae velit dignissim ornare.
    Vestibulum eu augue et nunc viverra placerat.
    Curabitur consectetur ante et ante pellentesque, non lacinia urna tincidunt.

    Duis in quam in ante pharetra imperdiet sit amet a tellus.
    Morbi aliquam mauris in magna faucibus lacinia.
    Curabitur vitae enim fringilla, commodo nunc ut, imperdiet felis.

    Pellentesque ac ipsum eget velit iaculis aliquam.
    Aenean nec orci vel dui fermentum sodales.
    Sed non metus vel dui laoreet mollis.
    Vestibulum sodales nisi ac nunc fermentum, sit amet convallis tellus varius.
    Fusce quis elit rutrum, suscipit nulla a, cursus nisl.

    Morbi interdum mauris id auctor euismod.

    Proin vel ligula vitae odio rutrum accumsan.
    Nullam malesuada velit finibus purus cursus, vel iaculis felis tincidunt.

    Sed a felis a metus iaculis accumsan ac a justo.
    Fusce ut sem a lorem placerat tincidunt in id neque.
    Nunc mollis erat a libero tristique, quis elementum lectus vehicula.
    Duis at dui sit amet magna scelerisque ultricies.
    Praesent quis erat vestibulum, iaculis arcu a, dignissim urna.

    Etiam maximus mauris a nisl aliquam egestas.
    Vivamus malesuada felis sit amet tempor sodales.

    Nunc in magna tincidunt, sollicitudin nisl non, auctor ex.
    Integer et magna a quam tempus euismod quis a felis.
    Cras sed mauris imperdiet, finibus neque non, cursus tortor.
    Aenean dapibus augue et elit tempus dictum et sit amet leo.

    Vivamus ullamcorper eros eu neque tempor, sed varius purus fermentum.
    Morbi eget turpis consequat, posuere tortor cursus, consequat magna.
    Quisque sed quam non risus tempus congue convallis nec nunc.
    Ut efficitur sem sit amet eros imperdiet, eget lobortis lorem blandit.
    Donec a tortor vitae elit commodo maximus quis vitae est.

    Curabitur luctus tortor nec ante tempus laoreet.

    Mauris aliquam dolor et blandit lacinia.
    Duis rutrum sem quis nisl mollis, pretium gravida dolor efficitur.
    Quisque nec tellus et ante blandit vulputate ac at orci.
    Etiam viverra neque vel sodales fringilla.
    Fusce auctor sapien eget sollicitudin gravida.

    Mauris pulvinar libero eu erat ullamcorper pretium.
    Sed tristique est non lorem facilisis, sit amet lobortis odio molestie.
    Sed consequat leo quis eros auctor, eget vulputate lectus cursus.
    Suspendisse euismod purus a convallis lobortis.
    Donec congue libero vitae erat scelerisque tincidunt.
    Morbi eu orci hendrerit erat ornare egestas fringilla a orci.

    Vivamus ac erat ac orci ornare luctus quis viverra arcu.

    Vivamus tincidunt arcu malesuada, ornare arcu ornare, tincidunt erat.
    Etiam luctus urna in ipsum aliquam dignissim.
    Sed interdum felis ut placerat ornare.
    Suspendisse lobortis elit ut bibendum sagittis.
    Ut non nibh nec mauris venenatis porta.

    Integer faucibus odio vitae interdum vulputate.
    Maecenas sodales eros vel pellentesque aliquam.

    Nunc ac felis tincidunt, eleifend turpis eu, fermentum nulla.
    Curabitur placerat mauris sit amet justo ullamcorper imperdiet.
    Duis quis enim a massa accumsan elementum in non augue.

    In eget dolor sed metus dictum gravida.
    Duis tincidunt nibh vitae odio lacinia, tempor facilisis nibh varius.
    Curabitur hendrerit augue eu lacus bibendum posuere.
    Vestibulum at nisi a nisi gravida blandit vitae eget arcu.

    Cras ultricies erat eget elit porttitor, at porta quam egestas.
    Proin placerat purus at quam sagittis bibendum.
    Nulla laoreet elit sit amet diam rutrum, sit amet tempus odio semper.
    Fusce ac libero laoreet leo varius sollicitudin nec quis leo.
    Duis semper tortor ac dui ultricies, in gravida nisi finibus.
    Donec ut nisl sed nibh tempor lacinia vel non diam.

    Aenean vehicula massa vel odio euismod commodo.
    Sed at lacus eget massa sollicitudin feugiat at in velit.
    Morbi vel lorem at risus convallis viverra et a massa.
    Nulla fermentum eros et iaculis semper.

    In in quam malesuada, condimentum justo at, aliquet sem.
    Morbi ullamcorper augue et lorem ornare, sit amet ornare leo suscipit.

    Etiam tempus libero sed feugiat egestas.
    Maecenas ut magna nec arcu ultricies tempor sit amet quis tellus.
    Nullam eget justo condimentum magna blandit ullamcorper.

    Donec pulvinar lectus ut consectetur pretium.
    Integer gravida urna in ligula consequat accumsan.

    Nam ac ex eu ex pellentesque condimentum.

    Integer auctor nulla sit amet purus ornare, in tincidunt nunc tristique.
    Donec efficitur tellus vel vestibulum sodales.
    Duis quis felis et purus elementum varius in ac neque.

    Nullam tempor risus in rhoncus scelerisque.
    Proin condimentum magna sed nisi vestibulum, in aliquet elit finibus.
    Nam hendrerit erat sit amet neque sodales, sed posuere velit gravida.

    Aenean malesuada ante eu ullamcorper pulvinar.

    Pellentesque scelerisque quam id ex pulvinar sagittis.
    Aenean nec ante nec elit facilisis semper at et massa.
    Vivamus eget lorem sit amet turpis tincidunt maximus ac et elit.

    Donec nec dui at est faucibus rutrum.
    Ut eleifend tortor tempor urna suscipit, ut sollicitudin enim mollis.
    Phasellus pharetra tortor eget neque dignissim congue.
    Integer eget dui id lorem luctus sollicitudin.
    Maecenas pellentesque nulla et mattis convallis.

    Nullam sed arcu euismod, pellentesque mauris ac, sollicitudin ligula.
    Curabitur interdum leo eu neque porttitor, vitae maximus enim suscipit.
    Sed id enim sit amet purus volutpat dapibus non vel magna.
    Sed id nisi quis leo consequat tincidunt.
    Morbi fermentum odio ac nisl cursus, at dictum nisl ultricies.

    Etiam et nisl ut sem tempor tempus.
    Quisque convallis nulla at pretium venenatis.
    Praesent auctor lacus id lacus mattis iaculis sed vitae diam.
    Vestibulum tempus magna ut nisl suscipit, vitae mattis dui mollis.

    Nam et dui sed orci pharetra lacinia.
    Proin a turpis pulvinar, malesuada odio ut, iaculis lacus.

    Phasellus at metus sit amet tellus porta blandit efficitur eu lectus.
    Nulla fringilla nibh quis sapien porta, condimentum commodo enim tristique.

    Curabitur accumsan quam eu lacus tempor posuere.
    Aenean rutrum ante a arcu euismod euismod.
    Fusce iaculis erat a lacus commodo, a maximus metus fringilla.
    Quisque sed dolor ut sapien tempor bibendum.
    Etiam ultrices sapien id velit suscipit, ut maximus libero ornare.

    Praesent semper arcu sed faucibus condimentum.
    Suspendisse nec neque ac nisl mollis semper.
    Curabitur accumsan mauris in ex hendrerit, non pretium lacus commodo.
    Nulla commodo urna eu orci interdum vehicula.
    Fusce convallis velit eget dui feugiat sollicitudin.
    Praesent quis nunc finibus, dictum enim vel, ultrices metus.

    Aliquam id mi ut leo venenatis condimentum.
    Maecenas mollis diam et nunc ultrices posuere.

    Ut efficitur mi non leo malesuada varius.

    Donec accumsan urna vel massa semper gravida.
    Nullam vehicula ligula elementum commodo mattis.
    Phasellus sed est vestibulum, consequat massa vitae, imperdiet nunc.
    Donec convallis nunc vitae interdum maximus.
    Ut lacinia dolor a condimentum pellentesque.

    Cras eget leo a diam elementum ullamcorper.

    Suspendisse consequat nisi eget nisl posuere porta.
    Proin eget sapien ut mi vulputate scelerisque vitae dictum sapien.
    Praesent accumsan justo non risus accumsan, ac vestibulum enim euismod.
    Ut at risus vitae est venenatis vestibulum eu finibus risus.

    Integer euismod purus vitae lorem maximus luctus.
    Aenean imperdiet elit sit amet leo ultrices, vel facilisis eros vulputate.
    Proin a turpis nec neque sagittis faucibus.
    Integer non felis sed odio vestibulum tristique.
    Aliquam pretium nibh vel molestie vulputate.

    Morbi condimentum mi a elementum lacinia.
    Integer ut lectus vitae quam hendrerit dictum nec at leo.
    Phasellus vel leo suscipit, blandit ligula nec, malesuada tellus.

    Vestibulum ullamcorper tortor vel commodo tempor.
    In id lectus nec ante tristique pretium.

    Vivamus laoreet erat at metus porta, et tristique nunc pellentesque.
    Vivamus cursus neque vel justo vehicula viverra.

    Sed in nulla in nunc interdum placerat ac vitae tellus.
    In suscipit ante blandit dapibus condimentum.
    Nam facilisis sem eget nibh consectetur fringilla.
    Donec ullamcorper felis sed vulputate eleifend.
    Duis vestibulum lacus at sapien sollicitudin viverra.
    Vestibulum vestibulum lorem id velit molestie egestas.

    In id tortor eu velit semper condimentum.
    Maecenas mattis ipsum id ullamcorper vulputate.
    Aenean auctor orci eget pharetra placerat.
    Aliquam accumsan lacus nec ullamcorper sodales.

    Etiam tincidunt turpis ut lorem pellentesque, sit amet volutpat velit sodales.
    Donec eleifend ligula non risus tincidunt, vel finibus urna varius.

    Pellentesque porttitor lacus at augue porttitor tempor.
    Nullam semper ex non diam consectetur consectetur.
    Nullam facilisis mi at felis sollicitudin malesuada.
    Vestibulum ac eros nec augue posuere viverra.
    Nulla sed nisl posuere, luctus ligula vitae, mollis sem.
    Pellentesque imperdiet nisl a metus tempor, laoreet ultrices metus blandit.

    Integer ullamcorper urna a luctus ornare.
    Aenean pharetra enim maximus est sollicitudin aliquet.
    Aliquam tincidunt est sed velit faucibus tempus.
    Donec a tortor ut dui tempor cursus sit amet sit amet ipsum.

    In hendrerit nulla ullamcorper varius faucibus.
    Sed aliquam mauris quis mauris ultrices, ut finibus elit dictum.
    Donec pulvinar odio vitae massa ultricies feugiat.

    Donec facilisis neque eget malesuada aliquet.
    Etiam sit amet elit non nulla ullamcorper imperdiet.
    Cras feugiat ligula eleifend neque lacinia, id ornare mauris placerat.
    Proin accumsan urna viverra, pharetra ante sit amet, volutpat quam.

    Duis vitae massa id libero malesuada lobortis.
    Pellentesque fermentum magna quis facilisis pellentesque.
    Nunc eget urna pretium diam gravida feugiat.
    Cras malesuada ante id elit fringilla, a lacinia ante hendrerit.
    Cras vitae quam vel metus aliquet volutpat vitae vel arcu.

    Nunc et tortor venenatis, posuere tortor a, lobortis sem.
    Suspendisse eu dui vel tortor mollis dictum.

    Nullam ut nisi imperdiet, aliquet leo sit amet, maximus augue.
    Cras nec purus et ex hendrerit dictum pharetra et ligula.
    In ut lorem eget libero auctor pretium.
    Sed vitae nisl aliquet, ultrices nisl quis, viverra urna.
    Nunc vitae enim sit amet est dapibus sodales eu sit amet sapien.
    Etiam at turpis a neque malesuada vestibulum in in justo.

    Aenean quis ante vel lacus commodo tristique.
    Curabitur a lorem eget urna maximus egestas ac eget augue.
    Aliquam id justo at nisl auctor rhoncus.
    Maecenas in mauris et neque dictum congue ac vitae tortor.
    Proin vitae nulla a dui rhoncus volutpat.

    Morbi vel purus eget libero euismod vehicula quis at mauris.
    Proin eu neque eleifend, tincidunt ex tempus, hendrerit magna.
    Pellentesque viverra diam in enim tempor, eget lacinia magna fringilla.
    Nullam vitae eros scelerisque quam faucibus iaculis.

    Praesent volutpat est in ante vestibulum ultricies.
    Nulla ac lacus non magna mollis congue at non neque.
    Integer nec nibh ac nisi porttitor pellentesque accumsan vitae lectus.

    Aenean blandit mi in sapien facilisis consequat.

    Pellentesque fringilla nisl ac mollis malesuada.
    Proin aliquam erat quis libero cursus eleifend.

    Integer vehicula sem a lorem consequat mattis.
    Etiam tempus lectus quis lacus feugiat euismod.
    Donec eu enim id eros porta fermentum a ac ligula.

    Cras pharetra mauris non pellentesque consequat.
    Proin dignissim massa et fermentum malesuada.
    Maecenas et ipsum sit amet neque consectetur bibendum sit amet egestas justo.
    Integer congue libero quis nulla eleifend, at gravida nibh euismod.
    Etiam id sem nec orci vestibulum varius.

    Morbi eu nisl iaculis, condimentum urna quis, consectetur justo.
    Proin in massa vitae felis dictum tempus quis vitae libero.
    Proin molestie elit eget purus ullamcorper, vel scelerisque nibh tincidunt.
    Vivamus et sapien vitae libero finibus mattis sit amet sed mi.

    Vivamus malesuada leo eget ipsum dictum, et suscipit lectus dictum.
    Pellentesque et mauris bibendum, ullamcorper leo nec, placerat orci.

    Mauris sollicitudin dui vel nibh mattis, vitae maximus justo ullamcorper.
    Nam in mauris feugiat, blandit velit eu, accumsan metus.

    Sed id risus sed leo dictum condimentum eget nec nulla.
    Fusce tristique felis pellentesque nisi ullamcorper condimentum.
    Donec quis turpis eu diam volutpat hendrerit.
    Nam congue sapien lacinia, ullamcorper quam ac, aliquam purus.

    Duis eget nulla lacinia, congue urna id, vestibulum leo.
    Maecenas a mauris laoreet, congue nulla in, venenatis lacus.
    Maecenas et lacus congue, malesuada dui ut, fermentum erat.
    Fusce ullamcorper eros quis dictum ultricies.
    Nunc vel nisl ullamcorper, venenatis mauris eu, viverra metus.
    Cras dignissim risus in nisl rhoncus, ut interdum ex sagittis.

    Sed nec diam pulvinar, sodales sem vitae, volutpat nibh.
    Vivamus mollis nunc sed risus tristique congue.

    Aliquam fringilla mauris quis justo volutpat ultricies.
    Nullam gravida sapien at libero suscipit, quis rhoncus velit laoreet.

    Sed lobortis lectus at mauris dictum, in facilisis nunc accumsan.
    Nunc hendrerit nisl eu tortor sagittis laoreet.
    Donec vestibulum enim quis lorem convallis imperdiet a eget eros.

    Nulla vel augue quis nisl consequat scelerisque at nec purus.
    In at sem eget ante tincidunt molestie a vitae ligula.
    Nullam eleifend felis et euismod hendrerit.

    In sit amet est vitae lectus finibus aliquam quis eget tortor.
    Vivamus sed urna in leo ullamcorper porttitor.
    Mauris id nunc quis felis molestie ornare.
    Pellentesque viverra nulla ut quam gravida consequat.
    Nulla facilisis odio quis feugiat varius.

    Mauris efficitur urna a lacus condimentum pretium.
    Phasellus ac diam tincidunt, semper lacus ut, porta sem.

    Nunc auctor tortor in condimentum aliquam.
    Nunc lobortis neque quis volutpat venenatis.
    Fusce quis lorem ac orci volutpat tempus eu ut velit.
    Phasellus nec quam nec libero faucibus sagittis quis nec urna.

    Morbi vitae leo malesuada, blandit lectus non, commodo lectus.
    Fusce condimentum felis a velit consequat semper.
    Fusce tincidunt odio quis bibendum malesuada.
    Praesent a odio a nulla pharetra fringilla.

    Mauris ut risus congue, feugiat libero ac, ullamcorper mauris.
    Mauris at urna cursus, dictum turpis at, auctor velit.

    Pellentesque finibus arcu ac lectus consectetur, eget laoreet erat porttitor.

    Maecenas in magna non justo aliquet semper nec sit amet nibh.
    Duis feugiat risus at dolor dapibus, quis euismod ex iaculis.
    Nullam tristique libero id ligula viverra dapibus.
    Nam congue risus ullamcorper nibh rhoncus, at euismod neque accumsan.
    Ut quis nisi et leo cursus maximus.

    Ut ac mauris dapibus, mollis ligula id, maximus urna.
    Nullam eget lacus fermentum magna lacinia convallis vitae sed felis.
    Cras non lectus aliquet, eleifend mauris sit amet, bibendum sem.

    Nunc nec magna sed odio lacinia feugiat.
    Pellentesque sit amet enim blandit ante finibus maximus.
    Pellentesque vel justo vel justo maximus vehicula aliquet eget nulla.

    Suspendisse consectetur lacus ut arcu malesuada, sit amet venenatis dolor molestie.
    Maecenas at elit id nunc hendrerit aliquam.
    Vestibulum lobortis dolor sit amet tempus auctor.
    Donec vel dolor pulvinar magna aliquet porta vel vitae arcu.
    Vestibulum sit amet libero eget erat suscipit venenatis.

    Nulla aliquet ante sed magna varius lacinia.
    Sed placerat mi sit amet mollis imperdiet.
    Sed venenatis felis at dolor interdum iaculis.

    Vivamus nec nunc ut odio finibus porta vitae in orci.
    Phasellus non ligula volutpat, sollicitudin lacus ut, luctus libero.
    Sed pellentesque purus a eros cursus tristique.

    Nullam at velit sit amet mauris vulputate sollicitudin vestibulum sit amet mi.
    Curabitur interdum dui interdum mi ullamcorper efficitur.
    Sed fringilla diam id est egestas, eu dictum libero luctus.
    Vestibulum ut ante sed metus dictum elementum.
    Pellentesque malesuada ante sit amet orci accumsan, in tempus elit sodales.

    Vivamus ac arcu pretium, tempor purus in, vulputate velit.
    Praesent a ex nec felis malesuada porttitor.

    Fusce vitae lectus vitae magna ullamcorper ultricies et sed ipsum.
    In vel risus placerat, scelerisque tortor sit amet, rhoncus velit.
    Integer sollicitudin nulla et nisi blandit dignissim.

    Curabitur at mauris scelerisque, vehicula ipsum ut, elementum est.
    Praesent ac urna facilisis, elementum lorem ac, consequat purus.

    Praesent non dolor eu ligula vehicula ullamcorper at convallis lorem.
    Praesent consequat nisi sed purus rhoncus dignissim.

    Maecenas ornare lorem at erat ullamcorper ultricies.
    Etiam eget purus id diam viverra accumsan.
    Mauris pretium tellus vitae varius ornare.

    Morbi iaculis libero id porta ornare.
    Nulla consequat tellus eget turpis faucibus ullamcorper.
    Donec venenatis velit sed libero convallis consectetur.
    In sit amet velit blandit, pharetra libero quis, dapibus quam.
    Sed quis eros efficitur, pulvinar tortor eget, lacinia sem.

    Ut nec eros ac arcu dapibus eleifend vel quis libero.
    Integer tincidunt tortor dapibus quam consequat vulputate.
    Nam efficitur augue eget elit ultricies, ac lobortis ligula pellentesque.
    Etiam non nisl id tortor tincidunt commodo eget vel arcu.
    Quisque tristique felis ac sodales cursus.
    Ut id elit vitae lacus fermentum dapibus at nec nibh.

    Aenean sed sapien id lacus lobortis sodales in nec eros.
    Quisque hendrerit nisi ut elit sollicitudin vulputate sed at nulla.
    Sed tincidunt nibh eget quam malesuada malesuada.
    Proin in diam sed sem finibus sodales.
    Pellentesque maximus eros eget cursus pretium.

    Duis tristique nisl sit amet est congue, vel fermentum metus sodales.
    Curabitur at ligula eget nulla tempor porttitor vitae non risus.
    Aenean finibus risus quis massa mollis, venenatis ultricies neque placerat.

    Maecenas in enim blandit, blandit tellus a, lacinia nisi.
    Praesent tristique risus nec dui tincidunt, quis sagittis felis malesuada.
    Curabitur eleifend nunc sit amet ex lacinia, a condimentum mi hendrerit.
    Etiam vestibulum mi at metus rutrum, nec aliquet nisl commodo.
    Quisque pulvinar lorem a interdum ultrices.

    Praesent cursus dui et nulla faucibus, vel pellentesque nunc ultricies.
    Integer cursus arcu ac urna porta, id dignissim elit malesuada.
    Morbi id lacus vel elit rutrum convallis.
    Nulla eleifend nisi at neque bibendum, sed placerat nisl volutpat.
    Vivamus eget sapien maximus, pharetra magna non, ultrices urna.

    Donec pulvinar tellus id leo tempus fringilla.

    Sed ut diam eu neque sagittis porta at nec nisl.
    Integer euismod nibh a euismod sollicitudin.
    Mauris facilisis mauris nec nunc suscipit bibendum.

    Ut rhoncus erat id vestibulum accumsan.
    Sed et magna ac sem auctor maximus.
    Maecenas egestas odio a eros egestas, non ultricies mi ultricies.
    Nunc eleifend enim quis aliquet feugiat.
    Duis at dolor a nibh luctus mollis non non nibh.
    Proin sit amet est vel ex cursus placerat a sit amet arcu.

    Sed a metus sollicitudin felis accumsan commodo.
    Ut porttitor mi sed tortor hendrerit fermentum.
    Nulla sagittis odio id libero maximus lobortis.
    Integer vitae augue commodo, rhoncus nibh non, egestas elit.
    Nulla malesuada sapien a aliquam luctus.

    In non ligula in erat sollicitudin gravida.
    Nam et mi vitae justo aliquet vehicula.
    Cras pellentesque ipsum eget lacus cursus, quis placerat lorem dapibus.
    Pellentesque tempus neque eget magna vulputate mollis.
    Aenean egestas neque sed elementum bibendum.
    Morbi rutrum ligula placerat tempor pellentesque.

    Ut eu diam placerat, tincidunt mauris sed, mattis justo.
    Fusce venenatis sem ut justo dignissim hendrerit.

    Nullam molestie odio non nisl ullamcorper hendrerit.
    Curabitur pellentesque ligula sed nunc vehicula consequat.
    Sed id sapien vulputate, volutpat quam ac, condimentum nisi.
    Duis volutpat elit ut maximus molestie.

    Integer eget nibh vel enim placerat elementum id eget massa.

    Quisque dapibus nisl non ligula ultrices cursus vitae at velit.
    Ut rhoncus dui quis mauris ullamcorper condimentum.
    Duis vel neque non nulla tincidunt tempus finibus id ex.

    Fusce ut felis fermentum ex dapibus aliquam.

    Suspendisse dictum odio ut sapien fringilla semper.

    Proin non mauris sed arcu imperdiet dictum ullamcorper at ipsum.
    In id nisl lobortis, maximus enim pretium, mollis lorem.
    Nulla id mauris pretium, convallis leo at, eleifend metus.

    Duis vitae erat sit amet orci posuere placerat vel eget augue.
    Ut lobortis augue vel sem venenatis euismod.

    Maecenas luctus nulla vel urna sagittis, eu congue libero consequat.
    Curabitur tristique lorem ut elit consectetur, a condimentum felis fermentum.
    Curabitur suscipit lorem et sapien gravida accumsan.

    Nulla porta ipsum non leo facilisis facilisis nec et magna.

    Nam cursus nunc vel dapibus vehicula.
    Praesent pulvinar justo id cursus aliquam.
    Praesent vitae tellus eu tellus feugiat accumsan.
    Etiam faucibus metus ut eros consequat malesuada.
    Phasellus a arcu eu erat tincidunt mattis finibus nec massa.

    Etiam porta sapien vitae vulputate vestibulum.
    Maecenas sed ante id neque vulputate imperdiet ac a ligula.
    Nulla semper mi ac dictum gravida.
    Etiam eu leo fermentum purus luctus convallis.

    Sed sit amet purus volutpat, eleifend justo ac, maximus nibh.
    Pellentesque sed libero commodo magna porttitor efficitur quis eu dolor.
    Etiam eu neque tempus, blandit lacus id, tristique tortor.
    Curabitur ultrices nunc mattis, molestie ante nec, euismod leo.
    Fusce ultricies felis sed rutrum feugiat.

    Fusce placerat lacus eu diam pulvinar, non laoreet dolor condimentum.
    Mauris vestibulum dolor et dui tristique, quis vulputate odio finibus.

    Sed tempor tellus ac diam vehicula, quis molestie mi sodales.
    Sed luctus erat nec maximus scelerisque.
    Vivamus vestibulum magna eu tincidunt pellentesque.

    Aenean dapibus elit ac aliquet sodales.
    Pellentesque malesuada nisi eget lorem eleifend, sit amet ultricies nisl luctus.
    Mauris molestie felis non faucibus pretium.
    In interdum velit at diam blandit, in pulvinar metus sodales.
    Curabitur ac velit et turpis hendrerit condimentum id non arcu.
    Phasellus viverra turpis eget magna egestas facilisis.

    Fusce nec velit id odio lacinia bibendum quis dapibus magna.
    Etiam vitae lacus tempor, gravida elit at, bibendum lacus.
    Quisque eget urna ullamcorper, lacinia est non, laoreet est.
    Vestibulum id nisi eget purus ullamcorper iaculis.
    Nulla accumsan arcu ac risus tristique, nec bibendum nibh iaculis.
    Pellentesque tempus libero sed nibh tincidunt, quis ultricies sapien iaculis.

    Suspendisse venenatis urna nec ante placerat luctus.
    Ut blandit ligula sit amet diam venenatis, vitae maximus libero aliquet.
    Nunc sit amet ligula id dui vehicula euismod ac et dui.
    Etiam finibus elit vel lorem accumsan, sit amet mattis ligula faucibus.

    Nam aliquet lectus vel tortor volutpat, sit amet porttitor purus vulputate.
    Vestibulum malesuada erat sed arcu vehicula molestie.
    Pellentesque ut mauris in purus convallis faucibus at et libero.
    Vestibulum venenatis magna id metus blandit posuere.
    Vestibulum ut turpis vitae libero volutpat pulvinar quis eu mi.
    Integer congue tortor id ligula pretium maximus.

    Fusce efficitur magna sit amet orci semper ultricies.

    Donec vestibulum enim vel augue luctus ornare.
    Suspendisse id ante posuere ex posuere fermentum in nec tellus.
    Proin aliquet urna at pellentesque ornare.
    Maecenas id magna sed orci varius sodales et in justo.

    Curabitur et dolor viverra, accumsan diam in, mattis neque.
    Proin pulvinar velit ac lorem rhoncus sodales.
    Integer nec lectus et elit pellentesque congue id ac sapien.
    Aenean vitae odio aliquam ex rhoncus volutpat non viverra magna.
    Sed id dui ac sapien bibendum faucibus.
    Sed at nisi vestibulum, egestas nunc eu, ultricies lacus.

    Fusce ornare nibh et nulla dapibus gravida.
    Fusce lobortis elit et justo ullamcorper mattis.

    Aliquam non ex sit amet nisl maximus ultricies.
    Etiam dictum massa nec tellus venenatis dignissim.

    Aenean nec risus viverra, fermentum velit vitae, auctor lectus.
    Donec accumsan lacus vel tempus condimentum.
    Nam tristique neque sollicitudin faucibus efficitur.
    Aenean sodales tortor nec sem suscipit accumsan.

    Pellentesque at purus bibendum, consequat nisi vitae, ullamcorper enim.
    Donec facilisis ipsum ut dolor facilisis, sed blandit nisl fringilla.
    Mauris cursus sapien vel dapibus commodo.
    Quisque id sapien nec sem pretium fermentum a ac ex.
    Nulla id massa sed urna commodo pellentesque.
    Pellentesque in sapien accumsan, ultrices libero et, suscipit eros.

    Vestibulum varius felis ac dui hendrerit laoreet ut a diam.
    Duis laoreet turpis elementum ligula pellentesque mollis.
    Morbi a dolor vel nisl malesuada interdum eget in metus.

    Curabitur vestibulum tellus ac massa semper luctus.
    Praesent nec arcu posuere, lobortis mauris ac, ornare quam.
    In viverra ipsum in est condimentum feugiat mollis at leo.

    Aliquam non turpis nec nisl blandit aliquet.
    Nam pretium metus eu neque fringilla rhoncus.
    Aenean bibendum sapien ac condimentum eleifend.
    Mauris consequat dui id odio molestie, vitae placerat eros mattis.
    Praesent aliquet dui vel ultricies convallis.

    Integer in sem et ante pharetra suscipit.
    Quisque quis nulla in neque facilisis consequat.
    Ut quis felis sit amet lacus viverra aliquam eget eu orci.

    Nulla porta lectus sed imperdiet maximus.

    Nam porttitor orci id elit congue, at egestas sapien ultricies.
    Nunc iaculis turpis ullamcorper ante commodo tristique.
    In pellentesque lectus gravida risus varius, non rhoncus purus faucibus.

    Quisque tempor est eget est sagittis, a tempus sapien placerat.
    Curabitur id augue imperdiet, iaculis tortor vitae, sodales nisl.
    Ut sit amet enim eget arcu rutrum suscipit.
    Pellentesque sed arcu mollis, varius tellus id, fringilla nulla.

    Phasellus dictum lacus ut ipsum auctor, at efficitur orci iaculis.
    Donec id tortor iaculis, rutrum leo quis, pulvinar metus.
    Vestibulum venenatis eros id ultrices sodales.
    Vestibulum at magna commodo, accumsan eros sit amet, viverra neque.
    Suspendisse at massa vitae elit congue mollis.
    Fusce varius sapien in arcu pharetra euismod.

    Vestibulum finibus purus ut convallis pharetra.
    Duis ac metus facilisis, mattis dolor id, pharetra arcu.
    Aliquam id felis quis nunc dictum faucibus.

    Suspendisse faucibus lectus sit amet sapien sodales, sed dapibus sem posuere.
    Sed a urna et arcu congue ullamcorper quis eget tortor.
    Cras a tellus sit amet sapien dictum ultricies eu sit amet leo.
    Sed ac enim fringilla, finibus eros vitae, molestie nisl.
    Aenean eu neque in sapien sodales gravida sit amet eu diam.
    Donec scelerisque ex dignissim molestie consequat.

    Cras et nisi ac augue euismod fermentum.
    Maecenas elementum augue sed sem eleifend rutrum.
    Proin interdum neque ac nibh sollicitudin placerat.

    Vestibulum ut nibh ornare, accumsan ipsum eget, condimentum nunc.
    Duis eget arcu nec ligula tincidunt ultrices.
    Praesent suscipit orci et justo posuere, sed bibendum diam faucibus.
    Etiam eu enim tristique, faucibus nibh quis, ultricies turpis.
    Donec eget velit vitae nisi venenatis efficitur.
    Morbi consectetur nisi eget iaculis sodales.

    Maecenas imperdiet odio ornare eros sagittis, eget efficitur eros venenatis.
    Praesent eget mi non magna placerat finibus vel non ipsum.
    Quisque euismod nibh a ex lobortis, eget aliquet nibh scelerisque.
    Curabitur et urna at enim fermentum pellentesque sagittis id mi.

    Morbi vitae felis hendrerit, molestie nisi at, posuere lectus.
    Nam dictum risus vel lorem suscipit, at venenatis ipsum blandit.

    Donec tincidunt nisl ac risus venenatis scelerisque.

    Phasellus varius est sed cursus egestas.
    Aliquam nec lorem lobortis, aliquet lectus eu, condimentum justo.
    Donec fringilla risus at dui bibendum, a condimentum tortor iaculis.
    Nulla rutrum arcu et pellentesque tincidunt.
    Nulla venenatis nisl in faucibus ullamcorper.

    Cras sed ante malesuada, scelerisque neque vel, tincidunt ante.
    Etiam at sem ullamcorper, suscipit metus vitae, posuere leo.

    Cras imperdiet felis mattis condimentum ornare.
    Donec egestas risus vitae dui maximus bibendum.
    Duis eget massa quis metus fermentum tincidunt.
    Aliquam tristique purus nec urna faucibus aliquam.

    Quisque non urna et sapien lobortis pellentesque nec eget eros."#;
