use std::rc::Rc;
use std::rc::Weak;
use std::mem::replace;
use std::cell::RefCell;

mod trie;



fn main() {
    // let test_text = "It was the best of times, it was the worst of times, it was the age of wisdom, it was the age of foolishness, it was the epoch of belief, it was the epoch of incredulity, it was the season of Light, it was the season of Darkness, it was the spring of hope, it was the winter of despair, we had everything before us, we had nothing before us, we were all going direct to Heaven, we were all going direct the other way – in short, the period was so far like the present period, that some of its noisiest authorities insisted on its being received, for good or for evil, in the superlative degree of comparison only.".to_lowercase();
    // let test_text = "test the test text".to_lowercase();
    // let test_text = "asasasasasasas".to_lowercase();
    let test_text = "Body is nothing more than emptiness, emptiness is nothing more than body.  The body is exactly empty, and emptiness is exactly body.  The other four aspects of human existence -- feeling, thought, will, and consciousness -- are likewise nothing more than emptiness, and emptiness nothing more than they.  All things are empty: Nothing is born, nothing dies, nothing is pure, nothing is stained, nothing increases and nothing decreases.  So, in emptiness, there is no body, no feeling, no thought, no will, no consciousness.  There are no eyes, no ears, no nose, no tongue, no body, no mind.  There is no seeing, no hearing, no smelling, no tasting, no touching, no imagining.  There is nothing seen, nor heard, nor smelled, nor tasted, nor touched, nor imagined.  There is no ignorance, and no end to ignorance.  There is no old age and death, and no end to old age and death.  There is no suffering, no cause of suffering, no end to suffering, no path to follow.  There is no attainment of wisdom, and no wisdom to attain.  The Bodhisattvas rely on the Perfection of Wisdom, and so with no delusions, they feel no fear, and have Nirvana here and now.  All the Buddhas, past, present, and future, rely on the Perfection of Wisdom, and live in full enlightenment.  The Perfection of Wisdom is the greatest mantra.  It is the clearest mantra, the highest mantra, the mantra that removes all suffering. This is truth that cannot be doubted.".to_lowercase();


    // string_to_trie(&test_text, Some(20)).borrow().print();
    trie::Trie::suffix(&test_text, Some(30)).print();

}




