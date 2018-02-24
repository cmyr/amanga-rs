extern crate manga_rs;
#[macro_use]
extern crate criterion;

use criterion::Criterion;
use manga_rs::{EditDistance, AsciiTester, Tester};

fn bench_edit_distance() -> usize {
    let mut tester = EditDistance::default();
    let inputs = [
        ("I’m so annoyed by him", "Hi my name is nobody"),
        ("Go for it. Fuck.", "fUCK I forgot"),
        ("mine istomorrow", "tomorrow is mine"),
        ("That shirt hurted.", "its the hard truth"),
        ("My cheeks need clappin", "Check my pinned please!"),
        ("Feel so unfair", "I suffer alone..."),
    ];
    let mut last_result = 0_usize;
    for &(one, two) in &inputs {
        last_result = tester.distance(one, two);
    }
    last_result
}

fn bench_is_match() -> bool {
    let inputs = vec![
        ("I am so over this.", "I am so over this! 😒"),
        ("I wish some nights lasted forever.", "I wish some nights lasted forever."),
        ("today i got a million, tomorrow i dont know", "Today I got a million, tomorrow I don't know🎶🎧"),
        ("You still like him, so stop lying to yourself.", "You still like him, so stop lying to yourself."),
        ("I hate my life so much 💀", "I HATE MY LIFE SO MUCH"),
        ("i want to be in LOVE !", "I want to be in love 🙃"),
        ("I just need you here", "I just need you here"),
        ("A mind troubled by doubt cannot focus on the course to victory. - Arthur Golden", "A mind troubled by doubt cannot focus on the course to victory. - Arthur Golden"),
        ("Our Wellness Doctors are highly experienced,trained by a specialist from Australia.Unique diagnostic methods are used for health evaluation.", "Our Wellness Doctors are highly experienced,trained by a specialist from Australia.Unique diagnostic methods are used for health evaluation."),
        ("DamnItsTrue\nfb\nTough time never last, but but but tough people do. .............. DamnItsTrue fb", "DamnItsTrue\nfb\nTough time never last, but but but tough people do. .............. DamnItsTrue fb"),
        ("pretamanger\nyummylish\nChristmas sandwiches have arrived in pretamanger this makes me happy yummylish", "pretamanger\nyummylish\nChristmas sandwiches have arrived in pretamanger this makes me happy yummylish"),
        ("Cause girl you're perfect, you're always worth it", "cause, girl, you're perfect, you're always worth it"),
        ("Fairytales dont always have a happy ending do they..", "fairy tales don't always have a happy ending, do they?"),
        ("what would you do? // by cityhigh", "What Would You Do by City High 🙌🏻"),
        ("i don't like this", "I don't like this"),
        ("There is no way to happiness, happiness is the way.", "THERE IS NO WAY TO HAPPINESS. HAPPINESS IS THE WAY."),
        ("I figure out you 😘 you figure out me 😘 we both a different breed 🤞🏾", "I figure out you, you figure out me we both a different breed"),
        ("I'm not perfect. I'll annoy you, make fun of you, say stupid things, but you'll never find someone who loves you as much as I do.", "I’m not perfect. I’ll annoy you, make fun of you, say stupid things, but you’ll never find someone who loves you as much as I do.:)"),
        ("I want to destroy everything.", "I want to destroy everything."),
        ("I'm thinking about it", "i’m thinking about it"),
        ("Next track: Maiak - Sometimes You've Got To Take the Hardest Line #NowPlaying #postrock", "Next track: Maiak - Sometimes You've Got To Take the Hardest Line #NowPlaying #postrock"),
        ("Attitudes are contagious. Is yours worth catching? - Bruce Van Horn #quote", "Attitudes are contagious. Is yours worth catching? - Bruce Van Horn #quote"),
        ("nobody cares about me", "Nobody cares about me 😔"),
        ("Missing you already 😔", "missing you already"),
        ("\"This is who I am. Nobody said you had to like it.\"", "This is who I am. Nobody said you had to like it 🙄🖕🏻"),
        ("Gyah! Wh-what do you want?!", "Gyah! Wh-what do you want?!"),
        ("It's half past eleven.\nT'eh lieh oor lurg nane jeig.\nIt's quarter to twelve.\nT'eh kerroo gys daa-yeig.", "It's half past eleven.\nT'eh lieh oor lurg nane jeig.\nIt's quarter to twelve.\nT'eh kerroo gys daa-yeig."),
        ("you dont know lol", "Lol you don’t know"),
        ("*sings along to The Eagles*\n\nKaraoke has changed my life!", "*sings along to The Eagles*\n\nKaraoke has changed my life!"),
        ("Bad day not a bad life", "Bad day not a bad life 😐"),
        ("wish I knew what's wrong with me.", "wish i knew whats wrong with me"),
        ("Maher Zain - Forgive Me", "Maher Zain - Forgive Me"),
        ("You're welcome.\nYou're very welcome.\nDon't mention it.\nNo problem.\nNo worries.\nOwa it ano man.\nIndi mo eon pagmitla-ngon.", "You're welcome.\nYou're very welcome.\nDon't mention it.\nNo problem.\nNo worries.\nOwa it ano man.\nIndi mo eon pagmitla-ngon."),
        ("How 'bout a round of applause👏🏻👏🏻👏🏻", "How ‘bout a round of applause 👏🏻"),
        ("Don't take this the wrong way but maybe this time we don't drive the truck.", "Don't take this the wrong way but maybe this time we don't drive the truck."),
        ("someone talk to me.", "Someone talk to me"),
        ("I just hope you miss me too.", "I just hope you miss me too..."),
        ("Onew Forever Love...", "Onew Forever Love..."),
        ("Why did the lightbulb cross the road? To get to the dark side.", "Why did the lightbulb cross the road? To get to the dark side."),
        ("My Heart by Paramore 👌💘", "My Heart by Paramore 😍😍"),
        ("Sometimes we expect to much from others, because we would be willing to do that much for them.", "Sometimes we expect to much from others, because we would be willing to do that much for them."),
        ("Like for something nice 🤘😊", "Like for something nice 🌕"),
        ("We lose ourselves in the things we love. We find ourselves there, too.", "We lose ourselves in the things we love. We find ourselves there, too."),
        ("its a beautiful day in the neighborhood", "it's a beautiful day in the neighborhood"),
        ("I feel unimportant..", "i feel unimportant"),
        ("Now playing pitbull - dont stop the party.mp3 by !", "Now playing pitbull - dont stop the party.mp3 by !"),
        ("Love truth, and pardon error. - Voltaire", "Love truth, and pardon error. - Voltaire"),
        ("It's in me lil nigga . I keep that semi lil nigga", "It's in me lil nigga . I keep that semi lil nigga"),
        ("Back like I never left", "Back like I never left 💯"),
        ("A lion doesn’t concern himself with the opinions of a sheep.", "A lion doesn't concern himself with the opinions of a sheep."),
        ("I love baseball", "i love baseball"),
        ("I'm not just sure, I'm HIV positive.", "I'm not just sure, I'm HIV positive."),
        ("Good morning..!", "Good morning☺️"),
        ("Friends: Can I come over? Real Friends: I'm coming over.", "Friends: Can I come over? Real Friends: I’m coming over.."),
        ("Take |silly staff\" photos. Have som. fun!", "Take \"silly staf.\" photos. Have somf fun!"),
        ("Headin' to Salt Lake City, Utah ? $50 Free Lyft credit w/ Lyft Coupon Code PIP #freeLyft #Lyftcoupon", "Headin' to Salt Lake City, Utah ? $50 Free Lyft credit w/ Lyft Coupon Code PIP #freeLyft #Lyftcoupon"),
        ("I know you playing games.", "I know you playing games🎧🎧"),
        ("\"He was no dragon. Fire cannot kill a dragon\"", "\"He was no dragon. Fire cannot kill a dragon.\""),
        ("I think a part of me will always be waiting for you", "I think a part of me will always be waiting for you."),
        ("Gimme another hour or two, hour with you", "gimme another hour or two, hour with you"),
        ("Have a purpose.", "Have a purpose."),
        ("\"I'm in pain, wanna put 10 shots in my brain\nI've been trippin' 'bout some things, can't change\nSuicidal, same time I'm tame\"", "I'm in pain, wanna put 10 shots in my brain\nI've been trippin' 'bout some things, can't change\nSuicidal, same time I'm tame"),
        ("I hate waiting", "I hate waiting 😒"),
        ("Otw to tagaytay", "Otw to Tagaytay📍"),
        ("It's been a hard day's night~ 🎶", "It's been a hard day's night"),
        ("It's always too good to be true", "It's always too good to be true👌🏽"),
        ("lowkey maybe highkey", "Low key. Maybe high key."),
        ("Sometimes music speaks what you feel inside.😌❣", "Sometimes music speaks what you feel inside."),
        ("I keep thinking I have school tomorrow😂", "I keep thinking i have school tomorrow"),
        ("Monster truck with big wheels. I am so angry. I want to burst into tears.", "Monster truck with big wheels. I am so angry. I want to burst into tears."),
        ("I hate being sick :(", "I hate being sick"),
        ("thinkin of a master plan 🤔", "Thinkin of a master plan"),
        ("When in doubt, tell the truth. - Mark Twain", "When in doubt, tell the truth. - Mark Twain"),
        ("I’m in pain , wanna put 10 shots to my brain", "I'm in pain, wanna put 10 shots to my brain"),
        ("And then one day you find ten years have got behind you\nNo one told you when to run, you missed the starting gun", "And then one day you find ten years have got behind you\nNo one told you when to run, you missed the starting gun"),
        ("I’m so annoyed by him", "Hi my name is nobody"),
        ("Go for it. Fuck.", "fUCK I forgot"),
        ("mine istomorrow", "tomorrow is mine"),
        ("That shirt hurted.", "its the hard truth"),
        ("My cheeks need clappin", "Check my pinned please!"),
        ("Feel so unfair", "I suffer alone..."),
        ];
    let mut tester = AsciiTester::default();
    let mut last_result = false;
    for &(one, two) in &inputs {
        last_result = tester.is_match(&one, &two);
    }
    last_result
}

fn edit_distance(c: &mut Criterion) {
    c.bench_function("edit_distance", |b| b.iter(|| bench_edit_distance()));
}

fn is_match(c: &mut Criterion) {
    c.bench_function("is_match", |b| b.iter(|| bench_is_match()));
}

criterion_group!(benches, edit_distance, is_match);
criterion_main!(benches);
