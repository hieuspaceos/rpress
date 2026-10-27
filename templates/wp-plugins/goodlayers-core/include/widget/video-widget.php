<?php
	/**
	 * A widget that show recent posts ( Specified by category ).
	 */

	add_action('widgets_init', 'gdlr_core_video_widget');
	if( !function_exists('gdlr_core_video_widget') ){
		function gdlr_core_video_widget() {
			register_widget( 'Goodlayers_Core_Video_Widget' );
		}
	}

	if( !class_exists('Goodlayers_Core_Video_Widget') ){
		class Goodlayers_Core_Video_Widget extends WP_Widget{

			// Initialize the widget
			function __construct() {

				parent::__construct(
					'gdlr-core-video-widget', 
					esc_html__('Video Widget ( Goodlayers )', 'goodlayers-core'), 
					array('description' => esc_html__('A widget that show video', 'goodlayers-core'))
				);  

			}

			// Output of the widget
			function widget( $args, $instance ) {
	
				$title = empty($instance['title'])? '': apply_filters('widget_title', $instance['title']);
				$video = empty($instance['video'])? '': $instance['video'];
				$thumbnail = empty($instance['thumbnail'])? '': $instance['thumbnail'];
				$thumbnail_size = empty($instance['thumbnail-size'])? 'full': $instance['thumbnail-size'];
					
				// Opening of widget
				echo gdlr_core_escape_content($args['before_widget']);
				
				// Open of title tag
				if( !empty($title) ){ 
					echo gdlr_core_escape_content($args['before_title'] . $title . $args['after_title']); 
				}
					
				// Widget Content
				if( !empty($video) ){
					echo '<div class="gdlr-core-video-widget gdlr-core-media-image" >';
					if( !empty($thumbnail) ){
						echo '<a ' . gdlr_core_get_lightbox_atts(array(
							'type' => 'video',
							'url' => $video
						)) . ' >';
						echo gdlr_core_get_image($thumbnail, $thumbnail_size); 
						echo '<i class="fa fa-play" ></i>';
						echo '</a>';
					}else{
						echo gdlr_core_get_video($video);
					}
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
						'video' => array(
							'type' => 'text',
							'id' => $this->get_field_id('video'),
							'name' => $this->get_field_name('video'),
							'title' => esc_html__('Video URL', 'goodlayers-core'),
							'value' => (isset($instance['video'])? $instance['video']: '')
						),
						'thumbnail' => array(
							'type' => 'upload',
							'id' => $this->get_field_id('thumbnail'),
							'name' => $this->get_field_name('thumbnail'),
							'title' => esc_html__('Video Thumbnail', 'goodlayers-core'),
							'value' => (isset($instance['thumbnail'])? $instance['thumbnail']: '')
						),
						'thumbnail-size' => array(
							'type' => 'combobox',
							'id' => $this->get_field_id('thumbnail-size'),
							'name' => $this->get_field_name('thumbnail-size'),
							'title' => esc_html__('Thumbnail Size', 'goodlayers-core'), 
							'options' => gdlr_core_get_thumbnail_list(),
							'value' => (isset($instance['thumbnail-size'])? $instance['thumbnail-size']: 'full')
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