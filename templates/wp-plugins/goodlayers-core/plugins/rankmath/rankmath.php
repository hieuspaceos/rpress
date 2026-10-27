<?php

	add_action('admin_enqueue_scripts', 'gdlr_core_rankmath_script'); 
	
	if( !function_exists('gdlr_core_rankmath_script') ){
		function gdlr_core_rankmath_script( $hook ){
			if( $hook == 'post.php' ){
				wp_enqueue_script('rank-math-goodlayers-core', GDLR_CORE_URL . '/plugins/rankmath/rankmath.js', array('wp-hooks', 'rank-math-analyzer'), false, true);
			}
		}
	}