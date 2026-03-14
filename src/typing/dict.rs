pub static DICT: &[&str] = &[
    "the", "be", "of", "and", "a", "to", "in", "he", "have", "it", "that", "for", "they", "I",
    "with", "as", "not", "on", "she", "at", "by", "this", "we", "you", "do", "but", "from", "or",
    "which", "one", "would", "all", "will", "there", "say", "who", "make", "when", "can", "more",
    "if", "no", "man", "out", "other", "so", "what", "time", "up", "go", "about", "than", "into",
    "could", "state", "only", "new", "year", "some", "take", "come", "these", "know", "see", "use",
    "get", "like", "then", "first", "any", "work", "now", "may", "such", "give", "over", "think",
    "most", "even", "find", "day", "also", "after", "way", "many", "must", "look", "before",
    "great", "back", "through", "long", "where", "much", "should", "well", "people", "down", "own",
    "just", "because", "good", "each", "those", "feel", "seem", "how", "high", "too", "place",
    "little", "world", "very", "still", "nation", "hand", "old", "life", "tell", "write", "become",
    "here", "show", "house", "both", "between", "need", "mean", "call", "develop", "under", "last",
    "right", "move", "thing", "general", "school", "never", "same", "another", "begin", "while",
    "number", "part", "turn", "real", "leave", "might", "want", "point", "form", "off", "child",
    "few", "small", "since", "against", "ask", "late", "home", "interest", "large", "person",
    "end", "open", "public", "follow", "during", "present", "without", "again", "hold", "govern",
    "around", "possible", "head", "consider", "word", "program", "problem", "however", "lead",
    "system", "set", "order", "eye", "plan", "run", "keep", "face", "fact", "group", "play",
    "stand", "increase", "early", "course", "change", "help", "line",
];
pub static QUOTE: &str = "You have the power to heal your life, and you need to know that.";

pub static article: &str = "In computer architecture, a branch predictor is a digital circuit that tries to guess which way a branch (e.g., an if–then–else structure) will go before this is known definitively. The purpose of the branch predictor is to improve the flow in the instruction pipeline. Branch predictors play a critical role in achieving high performance in many modern pipelined microprocessor architectures.

Two-way branching is usually implemented with a conditional jump instruction. A conditional jump can either be \"taken\" and jump to a different place in program memory, or it can be \"not taken\" and continue execution immediately after the conditional jump. It is not known for certain whether a conditional jump will be taken or not taken until the condition has been calculated and the conditional jump has passed the execution stage in the instruction pipeline.

Without branch prediction, the processor would have to wait until the conditional jump instruction has passed the execute stage before the next instruction can enter the fetch stage in the pipeline. The branch predictor attempts to avoid this waste of time by trying to guess whether the conditional jump is most likely to be taken or not taken. The branch that is guessed to be the most likely is then fetched and speculatively executed. If it is later detected that the guess was wrong, then the speculatively executed or partially executed instructions are discarded and the pipeline starts over with the correct branch, incurring a delay.

The time that is wasted in case of a branch misprediction is equal to the number of stages in the pipeline from the fetch stage to the execute stage. Modern microprocessors tend to have quite long pipelines so that the misprediction delay is between 10 and 20 clock cycles. As a result, making a pipeline longer increases the need for a more advanced branch predictor.

The first time a conditional jump instruction is encountered, there is not much information to base a prediction on. However, the branch predictor keeps records of whether or not branches are taken, so when it encounters a conditional jump that has been seen several times before, it can base the prediction on the recorded history. The branch predictor may, for example, recognize that the conditional jump is taken more often than not, or that it is taken every second time.

Branch prediction is not the same as branch target prediction. Branch prediction attempts to guess whether a conditional jump will be taken or not. Branch target prediction attempts to guess the target of a taken conditional or unconditional jump before it is computed by decoding and executing the instruction itself. Branch prediction and branch target prediction are often combined into the same circuitry.";
