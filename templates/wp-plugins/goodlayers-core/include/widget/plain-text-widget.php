<?php
	/**
	 * A widget that show recent posts ( Specified by category ).
	 */

	add_action('widgets_init', 'gdlr_core_plain_text_widget');
	if( !function_exists('gdlr_core_plain_text_widget') ){
		function gdlr_core_plain_text_widget() {
			register_widget( 'Goodlayers_Core_Plain_Text_Widget' );
		}
	}

	if( !class_exists('Goodlayers_Core_Plain_Text_Widget') ){
		class Goodlayers_Core_Plain_Text_Widget extends WP_Widget{

			// Initialize the widget
			function __construct() {

				parent::__construct(
					'gdlr-core-plain-text-widget', 
					esc_html__('Plain Text Widget ( Goodlayers )', 'goodlayers-core'), 
					array('description' => esc_html__('A widget that show plain text', 'goodlayers-core'))
				);  

			}

			// Output of the widget
			function widget( $args, $instance ) {
	
				$title = empty($instance['title'])? '': apply_filters('widget_title', $instance['title']);
				$content = empty($instance['content'])? '': $instance['content'];
				
				// Opening of widget
				echo gdlr_core_escape_content($args['before_widget']);
				
				// Open of title tag
				if( !empty($title) ){ 
					echo gdlr_core_escape_content($args['before_title'] . $title . $args['after_title']); 
				}

				if( !empty($content) ){ 
					echo '<span class="clear"></span>';
					echo '<div class="gdlr-core-plain-textwidget clearfix" >';
					echo gdlr_core_content_filter($content); 
					echo '</div>';
				}
				
				// Closing of widget
				echo gdlr_core_escape_content($args['after_widget']);

			}

			// Widget Form
			function form( $instance ) {

				if( class_exists('gdlr_core_widget_util') ){
					gdlr_core_widget_util::get_option(array(
						'title' => array(
							'type' => 'text',
							'id' => $this->get_field_id('title'),
							'name' => $this->get_field_name('title'),
							'title' => esc_html__('Title', 'goodlayers-core'),
							'value' => (isset($instance['title'])? $instance['title']: '')
						),
						'content' => array(
							'type' => 'textarea',
							'id' => $this->get_field_id('content'),
							'name' => $this->get_field_name('content'),
							'title' => esc_html__('Content', 'goodlayers-core'),
							'value' => (isset($instance['content'])? $instance['content']: '')
						),
					));
				}

			}
			
			// Update the widget
			function update( $new_instance, $old_instance ) {

				if( class_exists('gdlr_core_widget_util') ){
					return gdlr_core_widget_util::get_option_update($new_instance);
				}

				return $new_instance;
			}	
		} // class
	} // class_exists
?>