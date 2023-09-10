+++
title = "In defense of stale-bots"
date = 2023-07-27
updated = 2023-08-12
[taxonomies]
category = ["opensource"]
tags = ["opensource", "github", "automation"]
+++

You are a young developer. You're only getting started, and learning new things as you go, but you love the idea of telling computers what to do. *Automating* mundane tasks. It's really exciting to have this much freedom and control over something you own. Your only limit is your imagination!

Time goes by. You've come across GitHub quite a few times by now. A lot of code searches lead to GitHub projects created by other people. Other hobbyists, like you, doing what they love: *coding*.

Until now, you've been keeping your code to yourself. It's not really nice code. The code structure is rather poor. It's difficult to add new features. It has bugs. Boy does it have bugs. But it works for you. It makes your day-by-day a little nicer.

> "What if I just shared this with others?"

You've taken a lot of, er, inspiration, from open source projects over the past year. A lot of them are very useful! But not *quite* what you needed. On the other hand, *your* project solves *your* problems perfectly. By publishing your work, you would give back to that community. Share improvements. And others with a similar problem to yours would really value it!

So you do it. You wake up the next day, and the project seems to have piqued the interest of many. That's… really awesome! It's a nice feeling.

Not long after, pull requests start coming in. New feature additions, bug fixing, improving performance… And so do the issues.

You knew the project wasn't perfect. The code had certainly grown a lot, and not in a very nice way. But it worked. It had the features you needed. It was good enough.

You begin nicely answering questions in the issue tracker. Accepting most new features. Fixing bugs you never would've encountered yourself.

You start a new job as a junior programming. How exciting! And there you meet your significant-other. A wonderful person in all aspects. You spend a lot of time with them and learning in your job.

Time goes by. You're starting to grow a bit tired of this GitHub thing. The project has attracted *a lot* of attention. Many new issues are being created every day. Some extremely high quality! But plenty downright insulting you. You begin to close issues you know you won't work on.

This project wasn't your job. It never was. It was a hobby. A passion project. Which worked perfectly fine for you.

The constant notifications still bother you. You've disabled them. But the ever-growing issue count is always in the back of your mind. Thinking you need to bring it down. People *want* to use your work! You *want* to have a functional project! But you no longer have time.

You come across this "stale bot". It can *automate* the process of closing down issues! Being the automation-lover you are, you begin using it in a blink. Then, people can continue benefiting from your work, and you can stop worrying about having to deal with all the bug reports and reviews.

People are now angry their issue is closed without explanation. They're opening duplicates. Can't they see, this was just a hobby? This isn't your job! Why won't they realize this is *your* project? You don't have the time to fix all the issues. The stale bot is simply automating the process.

This isn't for you. The situation is frustrating and annoying to deal with. You take the project down. After all, it doesn't need to be online for you to make full use of it.

You did get feedback. You learnt a few things. The code is more performant, even if a bit harder to understand at places. There are new features, which… you don't really have a need for, but are cool, I guess. But you're glad you don't need to deal with the noise any longer.

---

> It is frustrating when you spend a lot of time on something for it to be ignored. Well-detailed bug reports, with stack traces, bisections pointing to the problem. Large patches for a new feature or a bug fix. There's progress, even if a little slow. Why can't the author have the decency of at least reading through?

It is probably not their job.

> Why did they post their work online if they were not willing to build a commuity?

They may have had a different goal in mind. They could simply want to share the work. Not deal with users, some of which have strong opinions and reflect it with equally strong language.

> There was a lot of high quality issues!

And a large amount of angry comments. Not everyone has such thick skin. It gets to you. Your brain remembers bad experiences.

> Couldn't they just lock the issues? Make it clear it's a won't-fix.

Maybe. The thought may not have crossed their mind. They may have weighed the pros and cons, and perhaps they wanted discussion to still happen, and take on the more interesting suggestions. They may only read through sparingly, picking up on a few. The rest would be closed automatically.

> Why did not they make their stance clear? Make it clear you're not accepting contributions.

Why should they? They're just sharing their work. They'll work on things they find appealing. They don't want to deal with codes of conduct, educating users, writing documentation, or fixing obscure bugs. They just wanted to share their work.

> They're willing to accept patches. Why not leave the issues open so others can pick them up?

Perhaps they just want an easy way to reduce clutter in the issues section. To keep only the ones the maintainer is actually interested in.

> The amount of issues wouldn't become a problem if they were triaged properly. Adding a tag is very easy!

But it is still a non-zero amount of effort. You still need to think about it. Read through. Actively do it. And they may simply have no desire to do so.

> The maintainer is just lazy. Malpractice.

But it's *their* issue tracker for *their* project. Wanting to keep a small amount of issues open is not necessarily evil. They may simply want to share code, not spend time moderating and triaging.

> The issue tracker is not only for the maintainer to use, it's for the users too!

That is not an universal truth. The maintainer gets the final say on how their project is run. If they want public issues and sparingly work on some on their free time, they can do so. If the project is large enough, unofficial communities can form elsewhere. The place to share knowledge does not necessarily need to be "official".

> The maintainer does not understand their responsibilities. They should hand over their hat if they're not willing to deal with issues properly.

The maintainer is simply sharing their work. There's no responsibility to keep issues open. This might be an unspoken "rule" of open source, but it's actually just a strong convention. The maintainer decides what "properly" means for their project. The existence of a stale bot should be clue enough on the maintainer's stance. The maintainer can keep sharing their work their way while forks exist.

> Closing issues as stale does not mean the issue is gone. They still exist.

Certainly. But when browsing through the issues, there is no denying that there is a lot less clutter. And if your goal is to take a look every now and then, it works great.

> Stale bots waste everyone's time. It's not always obvious that they are being used, and those interested need to actively keep the issues open to prevent automatic closing.

That's a choice the users make. The maintainer's stance is clear: the stale bot runs on the issue board. It is not the maintainer's problem that some users insist, even after it's clear that they maintainer doesn't care.

The lack of discoverability on whether stale bots are being used or not is a fair point. However, if the repository lacks contribution guidelines, you might want to spare a minute to check for the existence of a "stale" label, or do a quick search to figure out if issues are closed as "stale". By doing this, you don't need to commit to creating well-detailed reports if you're not willing to put up with the fact that it might just get closed without the author reading it.

> Stale bots create fatigue for everyone, as duplicate issues are created and those watching get pinged. This is rude on part of the maintainer.

Yes, this is annoying for the users. But not necessarily intentional malice from the maintainer. It is simply how they chose to automate a process. Open source has a few unspoken rules. Perhaps our junior developer was unaware of them. It is possible to educate them. Explain why you're against stale bots, and politely ask them to clarify what their stance is.

> Hobby projects are not an excuse to mistreat the community. Maintainers may not owe us support, but they owe us respect.

The maintainer may not have wanted to have a community in the first place. Social rules may say this behaviour is impolite. This is a fair point. Ask the maintainer to make it clear what their stance is. This question is uncomfortable, so it may happen that the topic is avoided. Whatever the maintainer's answer is, remember they have the final say on how they run their project. You should respect this choice, even if you strongly dislike it and it makes you uncomfortable. Their project might not be for you.

---

As unsatisfying as it is, it's a choice the maintainer can make. They may be aware of this problem and actively try to document how contributions are handled. But they don't have to. GitHub enables the issue tracker by default. The maintainer may want to keep it for their own use. They may not want to build a community.

Users have their own expectations. There's an issue tracker, so I can report bugs, right? Well, yes. You can report issues. But that doesn't mean the maintainer has to read them. And this does not make them a bad person.

Forking is always an option. Become a maintainer who is more engaged in building a community yourself. Triage the issues. This is not for everyone.

Stale bots are simply automating a task. One solution of many. And like every solution, it comes with trade-offs. Yes, it feels more "humane" for the maintainer to close the report as "won't-fix" with an explanation. Not everyone has time for that, or the temperament to do so. The maintainer does not necessarily feel "shame" for having a lot of issues open. They might simply want to keep a small amount of them open.

It's true. The maintainer may not care about your issues. But those are *your* issues, not *theirs*. They're free to spend their free time however they wish. And if they wish, they can choose to ignore and close said issues, while keeping a public project page up. If the maintainer's stance is not clear, take the opportunity to educate them on why it's important to make it clear. There are unspoken rules they may not be aware of.

If your expectations don't match those of the maintainer, don't get angry at the maintainer. You can consider their use of stale bots as their stance on the matter. You can disagree. Take a step back. Read the contribution rules and don't engage if you disagree. Save yourself a bad time.

To make it clear: this post's protagonist is a hobbyst. Of course, everyone's situation is different, and stale bots used by companies are a whole other matter. But don't go making other people's lives miserable. There's enough of that already. Some just want to code, not build a community. Harassing maintainers is never okay.

---

Personally, I haven't used stale bots before, and don't plan to in the near future. I haven't burnt out on answering issues just yet. (Plus, I find the whole process extremely dull, even if they're easy to "set up".)

When I was getting started in the open source world, I was thrilled to get bug reports and be able to answer. And not just bugs, but questions too. Code reviews, it was great!

I did eventually get tired. Some of my past answers were starting to be clearly impolite. I've been a dick to people. Sorry about that.

I've now realized angry answers help nobody, and try to at least provide some context, before closing issues myself if it's not actionable. The quality of my answer still depends on the quality of the question. But I really try to remain polite.

I think this is the best a maintainer can do. But I acknowledge it takes a lot of time, temper, dedication. And so, I can understand those who choose to use stale bots instead. They may not be aware of all the trade-offs and implications, but they can be educated.

Respect their choice, and keep it cool.
